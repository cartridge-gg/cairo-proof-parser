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

    assert_eq!(proof.config, proof_from_annotations.config);
    assert_eq!(proof.public_input, proof_from_annotations.public_input);
    assert_eq!(
        proof.unsent_commitment.oods_values.len(),
        proof_from_annotations.unsent_commitment.oods_values.len()
    );
    assert_eq!(
        proof.unsent_commitment.oods_values,
        proof_from_annotations.unsent_commitment.oods_values
    );
    assert_eq!(
        proof.unsent_commitment.traces,
        proof_from_annotations.unsent_commitment.traces
    );

    assert_eq!(
        proof.unsent_commitment.composition,
        proof_from_annotations.unsent_commitment.composition
    );

    assert_eq!(
        proof.witness.original_leaves.len(),
        proof_from_annotations.witness.original_leaves.len()
    );
    assert_eq!(
        proof.witness.original_leaves,
        proof_from_annotations.witness.original_leaves
    );
    assert_eq!(
        proof.witness.original_authentications.len(),
        proof_from_annotations
            .witness
            .original_authentications
            .len()
    );
    assert_eq!(
        proof.witness.original_authentications,
        proof_from_annotations.witness.original_authentications
    );

    assert_eq!(proof.witness, proof_from_annotations.witness);

    assert_eq!(proof, proof_from_annotations);

    println!("`hex_proof` is consistent with annotations.");

    Ok(())
}
