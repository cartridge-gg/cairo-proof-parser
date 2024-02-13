use cairo_proof_parser::lambda::{generate_proof_from_trace, write_proof_compatible_with_stone};
use clap::Parser;
use stark_platinum_prover::proof::options::{ProofOptions, SecurityLevel};

extern crate cairo_proof_parser;
extern crate clap;
extern crate stark_platinum_prover;

#[derive(Parser, Debug)]
struct Args {
    pub trace_bin_path: String,
    pub memory_bin_path: String,
    pub proof_path: String,
}

pub fn main() {
    let proof_options = ProofOptions::new_secure(SecurityLevel::Conjecturable100Bits, 3);
    let args = Args::parse();
    let Some((proof, pub_inputs)) =
        generate_proof_from_trace(&args.trace_bin_path, &args.memory_bin_path, &proof_options)
    else {
        return;
    };

    // write_proof(proof, pub_inputs, args.proof_path);
    write_proof_compatible_with_stone(proof, pub_inputs, args.proof_path, &proof_options)
}
