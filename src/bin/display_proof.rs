extern crate cairo_proof_parser;
use std::io::{self, Read};

use cairo_proof_parser::parse;

fn main() -> anyhow::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Parse the input as an AST
    let proof = parse(&input)?;

    println!("{proof}");
    Ok(())
}
