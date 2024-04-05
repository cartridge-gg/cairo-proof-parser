use std::{convert::TryFrom, fmt::Display};

use crate::{json_parser::ProofJSON, stark_proof::StarkProof};

mod annotations;
mod ast;
mod builtins;
mod json_parser;
mod layout;
mod stark_proof;
mod utils;

extern crate itertools;
extern crate num_bigint;
extern crate regex;
extern crate serde;

pub use ast::{Expr, Exprs};
use itertools::chain;

#[derive(Debug)]
pub struct ParseStarkProof {
    pub config: Exprs,
    pub public_input: Exprs,
    pub unsent_commitment: Exprs,
    pub witness: Exprs,
}

impl Display for ParseStarkProof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = chain![
            self.config.iter(),
            self.public_input.iter(),
            self.unsent_commitment.iter(),
            self.witness.iter()
        ];

        for (i, expr) in result.enumerate() {
            if i != 0 {
                write!(f, " ")?;
            }
            write!(f, "{expr}")?;
        }

        Ok(())
    }
}

pub fn parse(input: String) -> anyhow::Result<ParseStarkProof> {
    let proof_json = serde_json::from_str::<ProofJSON>(&input)?;
    let stark_proof = StarkProof::try_from(proof_json)?;
    Ok(ParseStarkProof {
        config: Exprs::from(stark_proof.config),
        public_input: Exprs::from(stark_proof.public_input),
        unsent_commitment: Exprs::from(stark_proof.unsent_commitment),
        witness: Exprs::from(stark_proof.witness),
    })
}
