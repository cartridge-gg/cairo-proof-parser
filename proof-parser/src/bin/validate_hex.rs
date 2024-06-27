use std::io::{self, Read};

use cairo_proof_parser::{
    json_parser::{proof_from_annotations, ProofJSON},
    parse,
};

fn main() -> anyhow::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let proof = parse(&input)?;

    let proof_json = serde_json::from_str::<ProofJSON>(&input)?;
    let proof_from_annotations = proof_from_annotations(proof_json)?;

    assert_eq!(proof, proof_from_annotations);

    Ok(())
}
