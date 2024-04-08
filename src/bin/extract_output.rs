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

    // Retrieve the output segment from the proof
    let output_segment = proof
        .public_input
        .segments
        .get(2)
        .ok_or_else(|| anyhow::Error::msg("Output segment not found"))?;

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

    // Extract program output using the address range in the output segment
    let program_output: Vec<FieldElement> = (output_segment.begin_addr..output_segment.stop_ptr)
        .map(|addr| {
            *main_page_map
                .get(&addr)
                .expect("Address not found in main page map")
        })
        .collect();

    // Calculate the Poseidon hash of the program output
    let output_hash = poseidon_hash_many(&program_output);

    let program_output_display: Vec<String> =
        program_output.iter().map(|x| x.to_string()).collect();
    let output_hash_display = output_hash.to_string();

    // Print the results
    println!("{:?}", program_output_display);
    println!("{}", output_hash_display);

    Ok(())
}
