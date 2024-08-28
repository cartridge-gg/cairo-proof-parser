use starknet_crypto::poseidon_hash_many;
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::convert::TryInto;

use crate::output::OUTPUT_SEGMENT_OFFSET;
use crate::parse_raw;

const PROGRAM_SEGMENT_OFFSET: usize = 0;

pub struct ExtractProgramResult {
    pub program: Vec<Felt>,
    pub program_hash: Felt,
}

pub fn extract_program(input: &str) -> anyhow::Result<ExtractProgramResult> {
    // Parse the input string into a proof structure
    let proof = parse_raw(input)?;

    // Retrieve the program segment from the proof
    let program_segment = proof
        .public_input
        .segments
        .get(PROGRAM_SEGMENT_OFFSET)
        .ok_or_else(|| anyhow::Error::msg("Program segment not found"))?;

    // Retrieve the execution segment from the proof
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

    let initial_pc = program_segment.begin_addr;

    // Extract program bytecode using the address range in the segments
    let program: Vec<Felt> = (initial_pc
        ..(proof.public_input.main_page.len() as u32 - output_segment.stop_ptr
            + output_segment.begin_addr))
        .map(|addr| {
            *main_page_map
                .get(&addr)
                .expect("Address not found in main page map")
        })
        .collect();

    // Calculate the Poseidon hash of the program output
    let program_hash = poseidon_hash_many(&program);

    Ok(ExtractProgramResult {
        program,
        program_hash,
    })
}
