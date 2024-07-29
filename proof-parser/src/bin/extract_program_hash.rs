use anyhow::Context;
use cairo_proof_parser::{program::ExtractProgramResult, StarkProof};
use std::io::{self, Read};

fn main() -> anyhow::Result<()> {
    // Read input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let proof = StarkProof::try_from(&input[..]).context("Failed to parse proof")?;
    let ExtractProgramResult {
        program: _,
        program_hash,
    } = proof.extract_program().unwrap();

    let program_hash_display = program_hash.to_string();

    println!("{program_hash_display}");

    Ok(())
}
