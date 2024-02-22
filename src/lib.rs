use std::convert::TryFrom;

use crate::{json_parser::ProofJSON, stark_proof::StarkProof};

mod annotations;
mod ast;
mod builtins;
mod json_parser;
mod layout;
mod stark_proof;
mod utils;

extern crate cairo_platinum_prover;
extern crate clap;
extern crate lambdaworks_math;
extern crate num_bigint;
extern crate regex;
extern crate serde;
extern crate stark_platinum_prover;

pub use ast::{Expr, Exprs};

pub fn parse(input: String) -> anyhow::Result<Exprs> {
    let proof_json = serde_json::from_str::<ProofJSON>(&input)?;
    let stark_proof = StarkProof::try_from(proof_json)?;
    Ok(Exprs::from(stark_proof))
}
