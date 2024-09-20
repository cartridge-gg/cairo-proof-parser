use cairo_proof_parser::{json_parser::proof_from_annotations, program::{CairoVersion, ExtractProgramResult}, ProofJSON};
use clap::Parser;
use std::io::{self, Read};

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Cli {   
    #[arg(short, long,default_value = "cairo")]
    cairo_version: CairoVersion,
}
fn main() -> anyhow::Result<()> {
    // Read input from stdin
    let args = Cli::parse(); 
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;


    let proof_json = serde_json::from_str::<ProofJSON>(&input)?;
    let proof_from_annotations = proof_from_annotations(proof_json)?;
    
    let ExtractProgramResult {
        program: _,
        program_hash,
    } = proof_from_annotations.extract_program(args.cairo_version).unwrap();

    let program_hash_display = program_hash.to_string();

    println!("{program_hash_display}");

    Ok(())
}
