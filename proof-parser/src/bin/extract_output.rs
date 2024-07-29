use anyhow::Context;
use cairo_proof_parser::{output::ExtractOutputResult, StarkProof};
use std::io::{self, Read};

fn main() -> anyhow::Result<()> {
    // Read input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let proof = StarkProof::try_from(&input[..]).context("Failed to parse proof")?;

    let ExtractOutputResult {
        program_output,
        program_output_hash,
    } = proof.extract_output().unwrap();

    let program_output_display: Vec<String> = program_output
        .iter()
        .map(std::string::ToString::to_string)
        .collect();
    let output_hash_display = program_output_hash.to_string();

    // Print the results
    println!("{program_output_display:?}");
    println!("{output_hash_display}");

    Ok(())
}
