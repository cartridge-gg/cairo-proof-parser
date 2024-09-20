use starknet_crypto::{poseidon_hash_many, Felt};

use crate::StarkProof;

const OUTPUT_SEGMENT_OFFSET: usize = 2;

pub struct ExtractProgramResult {
    pub program: Vec<Felt>,
    pub program_hash: Felt,
}

impl StarkProof {
    pub fn extract_program(&self) -> anyhow::Result<ExtractProgramResult> {
        let output_segment = self
            .public_input
            .segments
            .get(OUTPUT_SEGMENT_OFFSET)
            .ok_or_else(|| anyhow::Error::msg("Execution segment not found"))?;
        let output_len = output_segment.stop_ptr - output_segment.begin_addr;
        let main_len = self.public_input.main_page.len();
        // Based on https://github.com/HerodotusDev/integrity/blob/main/src/air/public_input.cairo#L180-L181
        let program = &self.public_input.main_page[0..main_len - output_len as usize].to_vec();

        let values: Vec<Felt> = program.iter().map(|cell| cell.value).collect();

        let hash = poseidon_hash_many(&values);

        Ok(ExtractProgramResult {
            program_hash: hash,
            program: values,
        })
    }
}
