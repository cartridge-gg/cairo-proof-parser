use std::{
    collections::{BTreeMap, HashMap},
    convert::TryFrom,
    vec,
};

use anyhow::anyhow;
use num_bigint::BigUint;
use serde::Deserialize;
use starknet_crypto::FieldElement;

use crate::{
    annotations::{extract::FromStrHex, Annotations},
    builtins::Builtin,
    deser::deser::{from_felts, from_felts_with_lengths, Deserializer},
    layout::Layout,
    stark_proof::{
        CairoPublicInput, FriConfig, FriLayerWitness, FriUnsentCommitment, FriWitness,
        ProofOfWorkConfig, ProofOfWorkUnsentCommitment, PublicMemoryCell, SegmentInfo, StarkConfig,
        StarkProof, StarkUnsentCommitment, StarkWitness, TableCommitmentConfig,
        TableCommitmentWitness, TableCommitmentWitnessFlat, TableDecommitment, TracesConfig,
        TracesDecommitment, TracesUnsentCommitment, TracesWitness, VectorCommitmentConfig,
        VectorCommitmentWitness, VectorCommitmentWitnessFlat,
    },
    utils::log2_if_power_of_2,
};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ProofJSON {
    proof_parameters: ProofParameters,
    annotations: Vec<String>,
    public_input: PublicInput,
    proof_hex: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ProofParameters {
    pub stark: Stark,
    #[serde(default)]
    pub n_verifier_friendly_commitment_layers: u32,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Stark {
    pub fri: Fri,
    pub log_n_cosets: u32,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Fri {
    pub fri_step_list: Vec<u32>,
    pub last_layer_degree_bound: u32,
    pub n_queries: u32,
    pub proof_of_work_bits: u32,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MemorySegmentAddress {
    begin_addr: u32,
    stop_ptr: u32,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct PublicMemoryElement {
    address: u32,
    page: u32,
    value: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct PublicInput {
    dynamic_params: Option<BTreeMap<String, BigUint>>,
    layout: Layout,
    memory_segments: HashMap<String, MemorySegmentAddress>,
    n_steps: u32,
    public_memory: Vec<PublicMemoryElement>,
    rc_min: u32,
    rc_max: u32,
}

pub fn bigint_to_fe(bigint: &BigUint) -> FieldElement {
    FieldElement::from_hex_be(&bigint.to_str_radix(16)).unwrap()
}

pub fn bigints_to_fe(bigint: &Vec<BigUint>) -> Vec<FieldElement> {
    bigint.iter().map(bigint_to_fe).collect()
}

impl ProofJSON {
    const COMPONENT_HEIGHT: u32 = 16;
    pub fn stark_config(&self) -> anyhow::Result<StarkConfig> {
        let stark = &self.proof_parameters.stark;
        let n_verifier_friendly_commitment_layers =
            self.proof_parameters.n_verifier_friendly_commitment_layers;

        let consts = match self
            .public_input
            .layout
            .get_dynamics_or_consts(&self.public_input.dynamic_params)
        {
            Some(c) => c,
            None => anyhow::bail!(
                "There were some constant overrides in the dynamic params but couldn't be parsed!"
            ),
        };

        let log_eval_domain_size = self.log_eval_damain_size()?;
        let traces = TracesConfig {
            original: TableCommitmentConfig {
                n_columns: consts.num_columns_first,
                vector: VectorCommitmentConfig {
                    height: log_eval_domain_size,
                    n_verifier_friendly_commitment_layers,
                },
            },
            interaction: TableCommitmentConfig {
                n_columns: consts.num_columns_second,
                vector: VectorCommitmentConfig {
                    height: log_eval_domain_size,
                    n_verifier_friendly_commitment_layers,
                },
            },
        };

        let composition = TableCommitmentConfig {
            n_columns: consts.constraint_degree,
            vector: VectorCommitmentConfig {
                height: log_eval_domain_size,
                n_verifier_friendly_commitment_layers,
            },
        };

        let fri = self.proof_parameters.stark.fri.clone();

        let proof_of_work = ProofOfWorkConfig {
            n_bits: fri.proof_of_work_bits,
        };
        let n_queries = fri.n_queries;

        let layer_log_sizes = self.layer_log_sizes()?;

        let fri_step_list = fri.fri_step_list;
        let log_last_layer_degree_bound = log2_if_power_of_2(fri.last_layer_degree_bound)
            .ok_or(anyhow::anyhow!("Invalid last layer degree bound"))?;
        let fri = FriConfig {
            log_input_size: layer_log_sizes[0],
            n_layers: fri_step_list.len() as u32,
            inner_layers: fri_step_list[1..]
                .iter()
                .zip(layer_log_sizes[2..].iter())
                .map(|(layer_steps, layer_log_rows)| TableCommitmentConfig {
                    n_columns: 2_u32.pow(*layer_steps),
                    vector: VectorCommitmentConfig {
                        height: *layer_log_rows,
                        n_verifier_friendly_commitment_layers,
                    },
                })
                .collect(),
            fri_step_sizes: fri_step_list,
            log_last_layer_degree_bound,
        };

        Ok(StarkConfig {
            traces,
            composition,
            fri,
            proof_of_work,
            log_trace_domain_size: self.log_trace_domain_size()?,
            n_queries,
            log_n_cosets: stark.log_n_cosets,
            n_verifier_friendly_commitment_layers,
        })
    }

    fn log_trace_domain_size(&self) -> anyhow::Result<u32> {
        let consts = self.public_input.layout.get_consts();
        let effective_component_height = Self::COMPONENT_HEIGHT * consts.cpu_component_step;
        log2_if_power_of_2(effective_component_height * self.public_input.n_steps)
            .ok_or(anyhow::anyhow!("Invalid cpu component step"))
    }

    fn log_eval_damain_size(&self) -> anyhow::Result<u32> {
        Ok(self.log_trace_domain_size()? + self.proof_parameters.stark.log_n_cosets)
    }

    fn layer_log_sizes(&self) -> anyhow::Result<Vec<u32>> {
        let mut layer_log_sizes = vec![self.log_eval_damain_size()?];
        for layer_step in &self.proof_parameters.stark.fri.fri_step_list {
            layer_log_sizes.push(layer_log_sizes.last().unwrap() - layer_step);
        }
        Ok(layer_log_sizes)
    }

    fn public_input(
        public_input: PublicInput,
        // z: BigUint,
        // alpha: BigUint,
    ) -> anyhow::Result<CairoPublicInput> {
        let continuous_page_headers = vec![];
        // Self::continuous_page_headers(&public_input.public_memory, z, alpha)?; this line does for now anyway
        let main_page = Self::main_page(&public_input.public_memory)?;
        let dynamic_params = public_input.dynamic_params.unwrap_or_default();
        let memory_segments = Builtin::sort_segments(public_input.memory_segments)
            .into_iter()
            .map(|s| SegmentInfo {
                begin_addr: s.begin_addr,
                stop_ptr: s.stop_ptr,
            })
            .collect::<Vec<_>>();
        let layout = BigUint::from_bytes_be(&public_input.layout.bytes_encode());
        let (padding_addr, padding_value) = match public_input.public_memory.first() {
            Some(m) => (
                m.address,
                BigUint::from_str_hex(&m.value).ok_or(anyhow::anyhow!("Invalid memory value"))?,
            ),
            None => anyhow::bail!("Invalid public memory"),
        };
        Ok(CairoPublicInput {
            log_n_steps: log2_if_power_of_2(public_input.n_steps)
                .ok_or(anyhow::anyhow!("Invalid number of steps"))?,
            range_check_min: public_input.rc_min,
            range_check_max: public_input.rc_max,
            layout,
            dynamic_params: dynamic_params.into_iter().collect(),
            n_segments: memory_segments.len(),
            segments: memory_segments,
            padding_addr,
            padding_value,
            main_page_len: main_page.len(),
            main_page,
            n_continuous_pages: continuous_page_headers.len(),
            continuous_page_headers,
        })
    }

    fn main_page(public_memory: &[PublicMemoryElement]) -> anyhow::Result<Vec<PublicMemoryCell>> {
        public_memory
            .iter()
            .filter(|m| m.page == 0)
            .map(|m| {
                Ok(PublicMemoryCell {
                    address: m.address,
                    value: BigUint::from_str_hex(&m.value)
                        .ok_or(anyhow::anyhow!("Invalid memory value"))?,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }

    fn _continuous_page_headers(
        _public_memory: &[PublicMemoryElement],
        _z: BigUint,
        _alpha: BigUint,
    ) -> anyhow::Result<Vec<BigUint>> {
        //TODO: Do it properly
        Ok(vec![])
    }

    fn stark_unsent_commitment(&self, annotations: &Annotations) -> StarkUnsentCommitment {
        StarkUnsentCommitment {
            traces: TracesUnsentCommitment {
                original: bigint_to_fe(&annotations.original_commitment_hash),
                interaction: bigint_to_fe(&annotations.interaction_commitment_hash),
            },
            composition: bigint_to_fe(&annotations.composition_commitment_hash),
            oods_values: bigints_to_fe(&annotations.oods_values),
            fri: FriUnsentCommitment {
                inner_layers: bigints_to_fe(&annotations.fri_layers_commitments),
                last_layer_coefficients: bigints_to_fe(&annotations.fri_last_layer_coefficients),
            },
            proof_of_work: ProofOfWorkUnsentCommitment {
                nonce: bigint_to_fe(&annotations.proof_of_work_nonce),
            },
        }
    }

    fn stark_witness(annotations: &Annotations) -> StarkWitness {
        StarkWitness {
            // traces_decommitment: TracesDecommitment {
            //     original_len: annotations.original_witness_leaves.len() as u32,
            //     original: bigints_to_fe(&annotations.original_witness_leaves),
            //     interaction: bigints_to_fe(&annotations.interaction_witness_leaves),
            // },
            traces_witness: TracesWitness {
                original: bigints_to_fe(&annotations.original_witness_authentications),
                skip: vec![],
                interaction: bigints_to_fe(&annotations.interaction_witness_authentications),
            },
            // composition_decommitment: TableDecommitment {
            //     n_values: annotations.composition_witness_leaves.len(),
            //     values: annotations.composition_witness_leaves.clone(),
            // },
            // composition_witness: TableCommitmentWitness {
            //     vector: VectorCommitmentWitness {
            //         n_authentications: annotations.composition_witness_authentications.len(),
            //         authentications: annotations.composition_witness_authentications.clone(),
            //     },
            // },
            // fri_witness: FriWitness {
            //     layers: annotations
            //         .fri_witnesses
            //         .iter()
            //         .map(|w| FriLayerWitness {
            //             n_leaves: w.leaves.len(),
            //             leaves: w.leaves.clone(),
            //             table_witness: TableCommitmentWitnessFlat {
            //                 vector: VectorCommitmentWitnessFlat {
            //                     n_authentications: w.authentications.len(),
            //                     authentications: w.authentications.clone(),
            //                 },
            //             },
            //         })
            //         .collect(),
            // },
        }
    }
}

#[derive(Debug)]
struct HexProof(Vec<FieldElement>);

impl TryFrom<&str> for HexProof {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> anyhow::Result<Self> {
        let hex: Vec<u8> = prefix_hex::decode(value).map_err(|_| anyhow!("Invalid hex"))?;
        let mut result = vec![];
        for chunk in hex.chunks(32) {
            result.push(FieldElement::from_byte_slice_be(chunk)?);
        }
        Ok(HexProof(result))
    }
}

impl TryFrom<ProofJSON> for StarkProof {
    type Error = anyhow::Error;
    fn try_from(value: ProofJSON) -> anyhow::Result<Self> {
        let config = value.stark_config()?;

        let hex = HexProof::try_from(value.proof_hex.as_str())?;

        let annotations = Annotations::new(
            &value
                .annotations
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            value.proof_parameters.stark.fri.fri_step_list.len(),
        )?;
        let public_input = ProofJSON::public_input(
            value.public_input.clone(),
            // annotations.z.clone(),
            // annotations.alpha.clone(),
        )?;
        let unsent_commitment = value.stark_unsent_commitment(&annotations);
        dbg!(unsent_commitment.oods_values.len());
        dbg!(unsent_commitment.fri.inner_layers.len());
        dbg!(unsent_commitment.fri.last_layer_coefficients.len());

        let witness = ProofJSON::stark_witness(&annotations);
        dbg!(witness.traces_witness.original.len());
        dbg!(witness.traces_witness.interaction.len());

        let deser: (StarkUnsentCommitment, Vec<FieldElement>, StarkWitness) =
            from_felts_with_lengths(
                &hex.0,
                vec![
                    Some(135),
                    Some(3),
                    Some(128),
                    Some(112), // skipped
                    Some(257),
                    Some(48), // skip
                    Some(257),
                ],
            )?;

        let mut deser_witness = deser.2;
        deser_witness.traces_witness.skip = vec![];

        assert_eq!(deser.0, unsent_commitment);
        assert_eq!(deser_witness, witness);

        Ok(StarkProof {
            config,
            public_input,
            unsent_commitment,
            // witness,
        })
    }
}
