use cairo_proof_parser::program::{extract_program, ExtractProgramResult};
use std::io::{self, Read};

fn main() -> anyhow::Result<()> {
    // Read input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let ExtractProgramResult {
        program: _,
        program_hash,
    } = extract_program(&input).unwrap();

    let program_hash_display = program_hash.to_string();

    println!("{program_hash_display}");

    Ok(())
}
