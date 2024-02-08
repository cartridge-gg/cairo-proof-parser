use std::convert::TryFrom;

use crate::{ast::Exprs, json_parser::ProofJSON, stark_proof::StarkProof};

mod annotations;
mod ast;
mod json_parser;
mod layout;
mod stark_proof;
mod utils;
mod builtins;

extern crate num_bigint;
extern crate serde;
extern crate regex;

pub fn parse(input: String) -> anyhow::Result<Exprs> {
    let proof_json = serde_json::from_str::<ProofJSON>(&input)?;
    let stark_proof = StarkProof::try_from(proof_json)?;
    Ok(Exprs::from(stark_proof))
}

