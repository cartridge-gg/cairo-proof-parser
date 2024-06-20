use ::serde::Deserialize;

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
pub struct ProverConfig {
    pub constraint_polynomial_task_size: u32,
    pub n_out_of_memory_merkle_layers: u32,
    pub table_prover_n_tasks_per_segment: u32,
}
