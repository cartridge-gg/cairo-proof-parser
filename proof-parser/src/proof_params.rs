use ::serde::Deserialize;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ProofParameters {
    pub stark: Stark,
    #[serde(default)]
    pub n_verifier_friendly_commitment_layers: u32,
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/main/verifier_main_helper_impl.cc#L54-L55#[derive(Deserialize, Debug, Clone, PartialEq)]
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
