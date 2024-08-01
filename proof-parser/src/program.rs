use starknet_crypto::{poseidon_hash_many, Felt};
use std::collections::HashMap;

use crate::StarkProof;

const PROGRAM_SEGMENT_OFFSET: usize = 0;
const OUTPUT_SEGMENT_OFFSET: usize = 2;

pub struct ExtractProgramResult {
    pub program: Vec<Felt>,
    pub program_hash: Felt,
}

impl StarkProof {
    pub fn extract_program(&self) -> anyhow::Result<ExtractProgramResult> {
        // Retrieve the program segment from the proof
        let program_segment = self
            .public_input
            .segments
            .get(PROGRAM_SEGMENT_OFFSET)
            .ok_or_else(|| anyhow::Error::msg("Program segment not found"))?;

        let output_segment = self
            .public_input
            .segments
            .get(OUTPUT_SEGMENT_OFFSET)
            .ok_or_else(|| anyhow::Error::msg("Execution segment not found"))?;
        let output_len = output_segment.stop_ptr - output_segment.begin_addr;

        // Extract program bytecode using the address range in the segments
        // Based on https://github.com/HerodotusDev/integrity/blob/bca869260c0c5d26bb18391356b095feb548aae5/src/air/public_input.cairo#L178-L179.
        let initial_pc = program_segment.begin_addr;
        let start = initial_pc + 1;
        let end = start + self.public_input.main_page.len() as u32 - output_len;

        // Construct a map for the main page elements
        let main_page_map = self
            .public_input
            .main_page
            .iter()
            .filter(|el| el.address >= start && el.address < end)
            .map(|el| (el.address, el.value))
            .collect::<HashMap<_, _>>();

        let program: Vec<Felt> = (start..end)
            .map(|addr| *main_page_map.get(&addr).unwrap_or(&Felt::ZERO))
            .collect();

        // Calculate the Poseidon hash of the program output
        let program_hash = poseidon_hash_many(&program);

        Ok(ExtractProgramResult {
            program,
            program_hash,
        })
    }
}
