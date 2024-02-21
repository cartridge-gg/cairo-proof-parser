use std::fmt::Display;

use serde::Deserialize;

// For now only the recursive and starknet layouts is supported
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    Recursive,
    Starknet,
}

impl Layout {
    pub fn get_consts(&self) -> LayoutConstants {
        match self {
            Layout::Recursive => LayoutConstants::recursive(),
            Layout::Starknet => LayoutConstants::starknet(),
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
            Layout::Starknet => write!(f, "starknet"),
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
    pub rc_builtin_ratio: usize,
    pub rc_n_parts: usize,
    pub ecdsa_builtin_ratio: usize,
    pub ecdsa_builtin_repetitions: usize,
    pub ecdsa_element_bits: usize,
    pub ecdsa_element_height: usize,
    pub ec_op_builtin_ratio: usize,
    pub ec_op_scalar_height: usize,
    pub ec_op_n_bits: usize,
    pub poseidon_ratio: usize,
    pub poseidon_m: usize,
    pub poseidon_rounds_full: usize,
    pub poseidon_rounds_partial: usize,
    pub memory_step: usize
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
            // Unused
            rc_builtin_ratio: 0,
            rc_n_parts: 0,
            ecdsa_builtin_ratio: 0,
            ecdsa_builtin_repetitions: 0,
            ecdsa_element_bits: 0,
            ecdsa_element_height: 0,
            ec_op_builtin_ratio: 0,
            ec_op_scalar_height: 0,
            ec_op_n_bits: 0,
            poseidon_ratio: 0,
            poseidon_m: 0,
            poseidon_rounds_full: 0,
            poseidon_rounds_partial: 0,
            memory_step: 0,
        }
    }
    pub fn starknet() -> Self {
        LayoutConstants {
            n_dynamic_params: 0,
            n_constraints: 195,
            mask_size: 269,
            public_memory_step: 8,
            has_diluted_pool: 1,
            diluted_spacing: 4,
            diluted_n_bits: 16,
            pedersen_builtin_ratio: 32,
            pedersen_builtin_repetitions: 1,
            rc_builtin_ratio: 16,
            rc_n_parts: 8,
            ecdsa_builtin_ratio: 2048,
            ecdsa_builtin_repetitions: 1,
            ecdsa_element_bits: 251,
            ecdsa_element_height: 256,
            bitwise_ratio: 64,
            bitwise_total_n_bits: 251,
            ec_op_builtin_ratio: 1024,
            ec_op_scalar_height: 256,
            ec_op_n_bits: 252,
            poseidon_ratio: 32,
            poseidon_m: 3,
            poseidon_rounds_full: 8,
            poseidon_rounds_partial: 83,
            has_output_builtin: 1,
            has_pedersen_builtin: 1,
            has_range_check_builtin: 1,
            has_ecdsa_builtin: 1,
            has_bitwise_builtin: 1,
            has_ec_op_builtin: 1,
            has_keccak_builtin: 0,
            has_poseidon_builtin: 1,
            layout_code: 0x737461726b6e6574,
            constraint_degree: 2,
            cpu_component_height: 16,
            log_cpu_component_height: 4,
            memory_step: 2,
            num_columns_first: 9,
            num_columns_second: 1,
            is_dynamic_air: 0,
            // Unused
            cpu_component_step: 0,
            pedersen_builtin_row_ratio: 0,
            range_check_builtin_ratio: 0,
            range_check_builtin_row_ratio: 0,
            range_check_n_parts: 0,
            bitwise_row_ratio: 0,
        }
    }
}
