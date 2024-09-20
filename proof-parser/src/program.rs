use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use starknet_crypto::{poseidon_hash_many, Felt};

use crate::StarkProof;

const OUTPUT_SEGMENT_OFFSET: usize = 2;
const EXECUTION_SEGMENT_OFFSET: usize = 1;
pub struct ExtractProgramResult {
    pub program: Vec<Felt>,
    pub program_hash: Felt,
}
#[derive(Debug, Clone, PartialEq,Serialize,Deserialize,ValueEnum)]
pub enum CairoVersion {
    Cairo,
    Cairo0,
}

impl StarkProof {
    pub fn extract_program(
        &self,
        cairo_version: CairoVersion,
    ) -> anyhow::Result<ExtractProgramResult> {
        let values = if cairo_version == CairoVersion::Cairo {
            self.extract_program_cairo()?
        } else {
            self.extract_program_cairo0()?
        };

        let hash = poseidon_hash_many(&values);

        Ok(ExtractProgramResult {
            program_hash: hash,
            program: values,
        })
    }
    fn extract_program_cairo(&self) -> anyhow::Result<Vec<Felt>> {
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
        Ok(values)
    }
    fn extract_program_cairo0(&self) -> anyhow::Result<Vec<Felt>> {
        let execution_segment = self
            .public_input
            .segments
            .get(EXECUTION_SEGMENT_OFFSET)
            .ok_or_else(|| anyhow::Error::msg("Execution segment not found"))?;

        let initial_fp = execution_segment.begin_addr;
        let program_end_pc = initial_fp - 2; // according to https://github.com/HerodotusDev/integrity/blob/main/src/air/layouts/recursive/public_input.cairo#L28-L48
        let program_len = program_end_pc - 1; // Subtract 1 to exclude the last cell, because list starts from 0 and main page is 1-indexed

        let program = &self.public_input.main_page[0..program_len as usize].to_vec();
        let values: Vec<Felt> = program.iter().map(|cell| cell.value).collect();
        Ok(values)
    }
}
