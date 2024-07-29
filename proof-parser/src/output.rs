use starknet_crypto::poseidon_hash_many;
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::convert::TryInto;

use crate::StarkProof;

const OUTPUT_SEGMENT_OFFSET: usize = 2;

pub struct ExtractOutputResult {
    pub program_output: Vec<Felt>,
    pub program_output_hash: Felt,
}

impl StarkProof {
    pub fn extract_output(&self) -> anyhow::Result<ExtractOutputResult> {
        // Retrieve the output segment from the proof
        let output_segment = self
            .public_input
            .segments
            .get(OUTPUT_SEGMENT_OFFSET)
            .ok_or_else(|| anyhow::Error::msg("Output segment not found"))?;

        // Extract program output using the address range in the output segment
        let output_len = output_segment.stop_ptr - output_segment.begin_addr;
        // Based on: https://github.com/HerodotusDev/integrity/blob/bca869260c0c5d26bb18391356b095feb548aae5/src/air/public_input.cairo#L182-L183.
        let start = self.public_input.main_page.len() - output_len as usize;
        let end = self.public_input.main_page.len();

        let program_output = self.public_input.main_page[start..end]
            .iter()
            .map(|cell| cell.value)
            .collect_vec();

        // Calculate the Poseidon hash of the program output
        let program_output_hash = poseidon_hash_many(&program_output);

        Ok(ExtractOutputResult {
            program_output,
            program_output_hash,
        })
    }
}
