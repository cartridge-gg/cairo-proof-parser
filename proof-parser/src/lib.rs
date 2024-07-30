use std::{convert::TryFrom, fmt::Display};

mod annotations;
mod builtins;
pub mod json_parser;
mod layout;
pub mod output;
pub mod program;
mod proof_params;
mod proof_structure;
mod stark_proof;
mod utils;

pub use crate::{json_parser::ProofJSON, stark_proof::StarkProof};
pub use serde_felt::{from_felts, to_felts};

impl Display for StarkProof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let serialized = to_felts(self).map_err(|_| std::fmt::Error)?;
        let done = serialized
            .into_iter()
            .map(|f| format!("{f}"))
            .collect::<Vec<_>>()
            .join(" ");

        write!(f, "{done}")?;

        Ok(())
    }
}

pub fn parse(input: &str) -> anyhow::Result<StarkProof> {
    let proof_json = serde_json::from_str::<ProofJSON>(input)?;
    let stark_proof = StarkProof::try_from(proof_json)?;

    Ok(stark_proof)
}
