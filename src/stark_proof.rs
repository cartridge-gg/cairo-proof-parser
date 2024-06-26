use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use starknet_crypto::FieldElement;

use crate::deser::montgomery::deserialize_montgomery_vec;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StarkProof {
    pub config: StarkConfig,
    pub public_input: CairoPublicInput<FieldElement>,
    pub unsent_commitment: StarkUnsentCommitment,
    pub witness: StarkWitnessReordered,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StarkConfig {
    pub traces: TracesConfig,
    pub composition: TableCommitmentConfig,
    pub fri: FriConfig,
    pub proof_of_work: ProofOfWorkConfig,
    pub log_trace_domain_size: u32,
    pub n_queries: u32,
    pub log_n_cosets: u32,
    pub n_verifier_friendly_commitment_layers: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TracesConfig {
    pub original: TableCommitmentConfig,
    pub interaction: TableCommitmentConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TableCommitmentConfig {
    pub n_columns: u32,
    pub vector: VectorCommitmentConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VectorCommitmentConfig {
    pub height: u32,
    pub n_verifier_friendly_commitment_layers: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FriConfig {
    pub log_input_size: u32,
    pub n_layers: u32,
    pub inner_layers: Vec<TableCommitmentConfig>,
    pub fri_step_sizes: Vec<u32>,
    pub log_last_layer_degree_bound: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProofOfWorkConfig {
    pub n_bits: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StarkUnsentCommitment {
    pub traces: TracesUnsentCommitment,
    pub composition: FieldElement,
    pub oods_values: Vec<FieldElement>,
    pub fri: FriUnsentCommitment,
    pub proof_of_work_nonce: FieldElement,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TracesUnsentCommitment {
    pub original: FieldElement,
    pub interaction: FieldElement,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FriUnsentCommitment {
    pub inner_layers: Vec<FieldElement>,
    pub last_layer_coefficients: Vec<FieldElement>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct StarkWitness {
    
    #[serde(deserialize_with = "deserialize_montgomery_vec")]
    pub original_leaves: Vec<FieldElement>,
    pub original_authentications: Vec<FieldElement>,
    #[serde(deserialize_with = "deserialize_montgomery_vec")]
    pub interaction_leaves: Vec<FieldElement>,
    pub interaction_authentications: Vec<FieldElement>,
    #[serde(deserialize_with = "deserialize_montgomery_vec")]
    pub composition_leaves: Vec<FieldElement>,
    pub composition_authentications: Vec<FieldElement>,
    pub fri_witness: FriWitness,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StarkWitnessReordered {
    #[serde(serialize_with = "double_len_serialize")]
    pub original_leaves: Vec<FieldElement>,
    #[serde(serialize_with = "double_len_serialize")]
    pub interaction_leaves: Vec<FieldElement>,
    #[serde(serialize_with = "double_len_serialize")]
    pub original_authentications: Vec<FieldElement>,
    #[serde(serialize_with = "double_len_serialize")]
    pub interaction_authentications: Vec<FieldElement>,
    #[serde(serialize_with = "double_len_serialize")]
    pub composition_leaves: Vec<FieldElement>,
    #[serde(serialize_with = "double_len_serialize")]
    pub composition_authentications: Vec<FieldElement>,
    pub fri_witness: FriWitness,
}

impl From<StarkWitness> for StarkWitnessReordered {
    fn from(witness: StarkWitness) -> Self {
        Self {
            original_leaves: witness.original_leaves,
            interaction_leaves: witness.interaction_leaves,
            original_authentications: witness.original_authentications,
            interaction_authentications: witness.interaction_authentications,
            composition_leaves: witness.composition_leaves,
            composition_authentications: witness.composition_authentications,
            fri_witness: witness.fri_witness,
        }
    }
}

pub fn double_len_serialize<S>(value: &Vec<FieldElement>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let value = VecWithLen {
        len: value.len(),
        vec: value.clone(),
    };
    value.serialize(serializer)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VecWithLen<T> {
    len: usize,
    vec: Vec<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FriWitness {
    pub layers: Vec<FriLayerWitness>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FriLayerWitness {
    #[serde(deserialize_with = "deserialize_montgomery_vec")]
    pub leaves: Vec<FieldElement>,
    pub table_witness: Vec<FieldElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CairoPublicInput<B> {
    pub log_n_steps: u32,
    pub range_check_min: u32,
    pub range_check_max: u32,
    pub layout: B,
    pub dynamic_params: BTreeMap<String, B>,
    pub n_segments: usize,
    pub segments: Vec<SegmentInfo>,
    pub padding_addr: u32,
    pub padding_value: B,
    pub main_page_len: usize,
    pub main_page: Vec<PublicMemoryCell<B>>,
    pub n_continuous_pages: usize,
    pub continuous_page_headers: Vec<B>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PublicMemoryCell<B> {
    pub address: u32,
    pub value: B,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SegmentInfo {
    pub begin_addr: u32,
    pub stop_ptr: u32,
}
