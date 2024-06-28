use starknet_crypto::poseidon_hash_many;
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::convert::TryInto;

use crate::parse_raw;

const OUTPUT_SEGMENT_OFFSET: usize = 2;

pub struct ExtractOutputResult {
    pub program_output: Vec<Felt>,
    pub program_output_hash: Felt,
}

pub fn extract_output(input: &str) -> anyhow::Result<ExtractOutputResult> {
    // Parse the input string into a proof structure
    let proof = parse_raw(input)?;

    // Retrieve the output segment from the proof
    let output_segment = proof
        .public_input
        .segments
        .get(OUTPUT_SEGMENT_OFFSET)
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
        let field_element =
            Felt::from_bytes_be(&padded_value.try_into().expect("Failed to convert to array"));

        main_page_map.insert(element.address, field_element);
    }

    // Extract program output using the address range in the output segment
    let program_output: Vec<Felt> = (output_segment.begin_addr..output_segment.stop_ptr)
        .map(|addr| {
            *main_page_map
                .get(&addr)
                .expect("Address not found in main page map")
        })
        .collect();

    // Calculate the Poseidon hash of the program output
    let program_output_hash = poseidon_hash_many(&program_output);

    Ok(ExtractOutputResult {
        program_output,
        program_output_hash,
    })
}
