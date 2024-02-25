use std::{collections::BTreeMap, convert::TryInto, fmt::Display};

use num_bigint::BigUint;
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
    pub fn get_dynamics_or_consts(
        &self,
        dynamic_params: &Option<BTreeMap<String, BigUint>>,
    ) -> Option<LayoutConstants> {
        let consts = self.get_consts();

        let dynamic_params =  match dynamic_params {
            Some(dp) => dp,
            None => return Some(consts)
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
}
