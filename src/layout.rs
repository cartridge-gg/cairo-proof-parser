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

pub(crate) struct LayoutConstants {
    pub cpu_component_step: u32,
    pub constraint_degree: u32,
    pub num_columns_first: u32,
    pub num_columns_second: u32,
}

impl LayoutConstants {
    pub fn recursive() -> Self {
        LayoutConstants {
            cpu_component_step: 1,
            constraint_degree: 2,
            num_columns_first: 7,
            num_columns_second: 3,
        }
    }
    pub fn starknet() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            num_columns_first: 9,
            num_columns_second: 1,
            // Unused
            cpu_component_step: 0,
        }
    }
}
