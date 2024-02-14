use std::convert::TryFrom;

use crate::json_parser::ProofJSON;
pub use crate::stark_proof::StarkProof;

mod annotations;
mod ast;
mod builtins;
mod json_parser;
mod layout;
mod stark_proof;
mod utils;

extern crate num_bigint;
extern crate regex;
extern crate serde;

pub use ast::{Expr, Exprs};

pub fn parse(input: String) -> anyhow::Result<Exprs> {
    let proof_json = serde_json::from_str::<ProofJSON>(&input)?;
    let stark_proof = StarkProof::try_from(proof_json)?;
    Ok(Exprs::from(stark_proof))
}

pub fn raw_parse(input: String) -> anyhow::Result<StarkProof> {
    let proof_json = serde_json::from_str::<ProofJSON>(&input)?;
    StarkProof::try_from(proof_json)
}
