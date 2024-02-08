use std::fmt::Display;

use serde::Deserialize;

// For now only the recursive layout is supported
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    Recursive,
}

impl Layout {
    pub fn get_consts(&self) -> LayoutConstants {
        match self {
            Layout::Recursive => LayoutConstants::recursive(),
        }
    }
    pub fn bytes_encode(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::Recursive => write!(f, "recursive"),
        }
    }
}

pub struct LayoutConstants {
    pub n_dynamic_params: u32,
    pub n_constraints: u32,
    pub mask_size: u32,
    pub cpu_component_step: u32,
    pub cpu_component_height: u32,
    pub public_memory_step: u32,
    pub has_diluted_pool: u32,
    pub diluted_spacing: u32,
    pub diluted_n_bits: u32,
    pub pedersen_builtin_ratio: u32,
    pub pedersen_builtin_row_ratio: u32,
    pub pedersen_builtin_repetitions: u32,
    pub range_check_builtin_ratio: u32,
    pub range_check_builtin_row_ratio: u32,
    pub range_check_n_parts: u32,
    pub bitwise_ratio: u32,
    pub bitwise_row_ratio: u32,
    pub bitwise_total_n_bits: u32,
    pub has_output_builtin: u32,
    pub has_pedersen_builtin: u32,
    pub has_range_check_builtin: u32,
    pub has_ecdsa_builtin: u32,
    pub has_bitwise_builtin: u32,
    pub has_ec_op_builtin: u32,
    pub has_keccak_builtin: u32,
    pub has_poseidon_builtin: u32,
    pub layout_code: u128,
    pub constraint_degree: u32,
    pub log_cpu_component_height: u32,
    pub num_columns_first: u32,
    pub num_columns_second: u32,
    pub is_dynamic_air: u32,
}

impl LayoutConstants {
    pub fn recursive() -> Self {
        LayoutConstants {
            n_dynamic_params: 0,
            n_constraints: 93,
            mask_size: 133,
            cpu_component_step: 1,
            cpu_component_height: 16,
            public_memory_step: 16,
            has_diluted_pool: 1,
            diluted_spacing: 4,
            diluted_n_bits: 16,
            pedersen_builtin_ratio: 128,
            pedersen_builtin_row_ratio: 2048,
            pedersen_builtin_repetitions: 1,
            range_check_builtin_ratio: 8,
            range_check_builtin_row_ratio: 128,
            range_check_n_parts: 8,
            bitwise_ratio: 8,
            bitwise_row_ratio: 128,
            bitwise_total_n_bits: 251,
            has_output_builtin: 1,
            has_pedersen_builtin: 1,
            has_range_check_builtin: 1,
            has_ecdsa_builtin: 0,
            has_bitwise_builtin: 1,
            has_ec_op_builtin: 0,
            has_keccak_builtin: 0,
            has_poseidon_builtin: 0,
            layout_code: 0x726563757273697665,
            constraint_degree: 2,
            log_cpu_component_height: 4,
            num_columns_first: 7,
            num_columns_second: 3,
            is_dynamic_air: 0,
        }
    }
}
