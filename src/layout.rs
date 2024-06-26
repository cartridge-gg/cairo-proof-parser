use std::{collections::BTreeMap, convert::TryInto, fmt::Display};

use num_bigint::BigUint;
use serde::Deserialize;

use crate::{
    json_parser::PublicInput,
    proof_params::{Fri, ProofParameters, Stark},
    utils::log2_if_power_of_2,
};

// For now only the recursive and starknet layouts is supported
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    Dex,
    Plain,
    Recursive,
    RecursiveWithPoseidon,
    Small,
    Starknet,
    StarknetWithKeccak,
}

impl Layout {
    pub fn get_consts(&self) -> LayoutConstants {
        match self {
            Layout::Dex => LayoutConstants::dex(),
            Layout::Plain => LayoutConstants::plain(),
            Layout::Recursive => LayoutConstants::recursive(),
            Layout::RecursiveWithPoseidon => LayoutConstants::recursive_with_poseidon(),
            Layout::Small => LayoutConstants::small(),
            Layout::Starknet => LayoutConstants::starknet(),
            Layout::StarknetWithKeccak => LayoutConstants::starknet_with_keccak(),
        }
    }
    pub fn get_dynamics_or_consts(
        &self,
        dynamic_params: &Option<BTreeMap<String, BigUint>>,
    ) -> Option<LayoutConstants> {
        let consts = self.get_consts();

        let Some(dynamic_params) = dynamic_params else {
            return Some(consts);
        };

        Some(LayoutConstants {
            cpu_component_step: dynamic_params
                .get("cpu_component_step")
                .map(<&BigUint>::try_into)
                .map(Result::ok)?
                .unwrap_or(consts.cpu_component_step),
            constraint_degree: dynamic_params
                .get("constraint_degree")
                .map(<&BigUint>::try_into)
                .map(Result::ok)?
                .unwrap_or(consts.constraint_degree),
            num_columns_first: dynamic_params
                .get("num_columns_first")
                .map(<&BigUint>::try_into)
                .map(Result::ok)?
                .unwrap_or(consts.num_columns_first),
            num_columns_second: dynamic_params
                .get("num_columns_second")
                .map(<&BigUint>::try_into)
                .map(Result::ok)?
                .unwrap_or(consts.num_columns_second),
        })
    }
    pub fn bytes_encode(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::Dex => write!(f, "dex"),
            Layout::Plain => write!(f, "plain"),
            Layout::Recursive => write!(f, "recursive"),
            Layout::RecursiveWithPoseidon => write!(f, "recursive_with_poseidon"),
            Layout::Small => write!(f, "small"),
            Layout::Starknet => write!(f, "starknet"),
            Layout::StarknetWithKeccak => write!(f, "starknet_with_keccak"),
        }
    }
}

pub(crate) struct LayoutConstants {
    pub cpu_component_step: u32,
    pub constraint_degree: u32,
    pub num_columns_first: u32,
    pub num_columns_second: u32,
}

impl LayoutConstants {
    pub fn recursive() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            cpu_component_step: 1,
            num_columns_first: 7,
            num_columns_second: 3,
        }
    }
    pub fn starknet() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            cpu_component_step: 1,
            num_columns_first: 9,
            num_columns_second: 1,
        }
    }
    pub fn small() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            cpu_component_step: 1,
            num_columns_first: 23,
            num_columns_second: 2,
        }
    }
    pub fn recursive_with_poseidon() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            cpu_component_step: 1,
            num_columns_first: 6,
            num_columns_second: 2,
        }
    }
    pub fn plain() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            cpu_component_step: 1,
            num_columns_first: 6,
            num_columns_second: 2,
        }
    }
    pub fn starknet_with_keccak() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            cpu_component_step: 1,
            num_columns_first: 12,
            num_columns_second: 3,
        }
    }
    pub fn dex() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            cpu_component_step: 1,
            num_columns_first: 21,
            num_columns_second: 1,
        }
    }
}

impl Layout {
    // https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/air/cpu/board/cpu_air_definition4.inl#L1775-L1776
    pub fn mask_len(&self) -> usize {
        match self {
            Layout::Recursive => 133,
            _ => unimplemented!(),
        }
    }
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/stark/committed_trace.cc#L212-L213
pub fn data_queries_len(public_input: &PublicInput) -> usize {
    let trace_len = trace_len(public_input.layout, public_input.n_steps);
    match public_input.layout {
        Layout::Recursive => 2,
        _ => unimplemented!(),
    }
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/air/cpu/board/cpu_air_definition4.h#L396-L397
pub fn trace_len(layout: Layout, n_steps: u32) -> u32 {
    match layout {
        Layout::Recursive => n_steps * 16 * 1,
        _ => unimplemented!(),
    }
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/stark/stark.cc#L164-L167
pub fn list_of_cosets(public_input: &PublicInput, proof_params: &ProofParameters) -> usize {
    1usize << proof_params.stark.log_n_cosets
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/stark/stark.cc#L204-L205
pub fn fft_bases(layout: Layout, n_steps: u32, proof_params: &ProofParameters) -> u32 {
    let trace_len = trace_len(layout, n_steps);
    let log_trace_len = log2_if_power_of_2(trace_len).unwrap();
    log_trace_len + proof_params.stark.log_n_cosets
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/fri/fri_verifier.cc#L104-L105
pub fn first_fri_layer(proof_params: &ProofParameters) -> u32 {
    proof_params.stark.fri.fri_step_list[0]
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/fri/fri_verifier.cc#L156-L157
pub fn inner_fri_layers(proof_params: &ProofParameters) -> Vec<(u32, u32)> {
    proof_params
        .stark
        .fri
        .fri_step_list
        .iter()
        .skip(1)
        .map(|&x| (x, 1 << x))
        .collect()
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/stark/stark.cc#L303-L304
pub fn fri_degree_bound(proof_params: &ProofParameters) -> u32 {
    let mut expected = proof_params.stark.fri.last_layer_degree_bound;
    for s in &proof_params.stark.fri.fri_step_list {
        expected *= 1 << s
    }
    expected
}

pub fn leaves(proof_params: &ProofParameters) -> Vec<usize> {
    proof_params
        .stark
        .fri
        .fri_step_list
        .iter()
        .skip(1)
        .map(|&x| (1u32 << x + 4) - 16)
        .map(|x| x as usize)
        .collect()
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/commitment_scheme/packaging_commitment_scheme.cc#L245-L250
pub fn authentications() -> usize {
    256 + 1
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProofStructure {
    pub first_layer_queries: usize,
    pub layer_count: usize,
    pub composition_decommitment: usize,
    pub oods: usize,
    pub composition_leaves: usize,
    pub last_layer_degree_bound: usize,
    pub authentications: usize,
    pub layer: Vec<usize>,
    pub witness: Vec<usize>,
}

impl ProofStructure {
    pub fn new(proof_params: &ProofParameters, layout: Layout, _n_steps: u32) -> Self {
        let n_queries = proof_params.stark.fri.n_queries;
        let mask_len = layout.mask_len();
        let layout = layout.get_consts();

        ProofStructure {
            // https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/stark/stark.cc#L276-L277
            first_layer_queries: (n_queries * layout.num_columns_first) as usize,

            layer_count: proof_params.stark.fri.fri_step_list.len() - 1,
            composition_decommitment: (n_queries * layout.num_columns_second) as usize,

            // https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/stark/oods.cc#L92-L93
            oods: mask_len + layout.num_columns_second as usize - 1,
            last_layer_degree_bound: proof_params.stark.fri.last_layer_degree_bound as usize,

            // https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/stark/composition_oracle.cc#L288-L289
            composition_leaves: 2 * n_queries as usize,
            authentications: authentications(),

            layer: leaves(proof_params),
            witness: witness(&proof_params.stark.fri),
        }
    }
}

pub fn witness(fri: &Fri) -> Vec<usize> {
    let first_fri_step = 16;
    let mut cumulative = 0;
    let mut vec = Vec::new();

    // https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/fri/fri_details.cc#L93-L97
    for v in fri.fri_step_list.iter().skip(1) {
        cumulative += *v;
        vec.push(first_fri_step - cumulative);
    }

    // https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/fri/fri_details.cc#L74-L82
    vec.into_iter()
        .map(|len| fri.n_queries * len + 1)
        .map(|x| x as usize)
        .collect()
}

#[test]
fn test_lens() {
    let n_steps = 16384;
    let layout = Layout::Recursive;
    let proof_params = ProofParameters {
        stark: Stark {
            fri: Fri {
                fri_step_list: vec![0, 4, 4, 3],
                last_layer_degree_bound: 128,
                n_queries: 16,
                proof_of_work_bits: 30,
            },
            log_n_cosets: 3,
        },
        n_verifier_friendly_commitment_layers: 0,
    };

    assert_eq!(fri_degree_bound(&proof_params), 262144);

    let result = ProofStructure::new(&proof_params, layout, n_steps);

    let expected = ProofStructure {
        first_layer_queries: 112,
        layer_count: 3,
        composition_decommitment: 48,
        oods: 135,
        last_layer_degree_bound: 128,
        composition_leaves: 32,
        authentications: 257,
        layer: vec![240, 240, 112],
        witness: vec![193, 129, 81],
    };
    assert_eq!(result, expected);
}
