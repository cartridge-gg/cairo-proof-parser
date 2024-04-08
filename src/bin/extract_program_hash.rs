extern crate cairo_proof_parser;
extern crate starknet_crypto;

use cairo_proof_parser::parse_raw;
use starknet_crypto::{poseidon_hash_many, FieldElement};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::{self, Read};

fn main() -> anyhow::Result<()> {
    // Read input from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    // Parse the input string into a proof structure
    let proof = parse_raw(input)?;

    // Retrieve the program segment from the proof
    let program_segment = proof
        .public_input
        .segments
        .get(0)
        .ok_or_else(|| anyhow::Error::msg("Program segment not found"))?;

    // Retrieve the execution segment from the proof
    let execution_segment = proof
        .public_input
        .segments
        .get(1)
        .ok_or_else(|| anyhow::Error::msg("Execution segment not found"))?;

    // Construct a map for the main page elements
    let mut main_page_map = HashMap::new();
    for element in &proof.public_input.main_page {
        let value_bytes = element.value.to_bytes_be();
        let padded_value = vec![0u8; 32 - value_bytes.len()]
            .iter()
            .chain(value_bytes.iter())
            .copied()
            .collect::<Vec<u8>>();
        let field_element = FieldElement::from_bytes_be(
            &padded_value.try_into().expect("Failed to convert to array"),
        )
        .expect("Failed to convert to FieldElement");

        main_page_map.insert(element.address, field_element);
    }

    let initial_pc = program_segment.begin_addr;
    let initial_fp = execution_segment.begin_addr;

    // Extract program bytecode using the address range in the segments
    let program: Vec<FieldElement> = (initial_pc..(initial_fp - initial_pc - 1))
        .map(|addr| {
            main_page_map
                .get(&addr)
                .expect("Address not found in main page map")
                .clone()
        })
        .collect();

    // Calculate the Poseidon hash of the program output
    let program_hash = poseidon_hash_many(&program);

    let program_hash_display = program_hash.to_string();

    // Print the results
    println!("{}", program_hash_display);

    Ok(())
}
