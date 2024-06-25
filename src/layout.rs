use std::{collections::BTreeMap, convert::TryInto, fmt::Display};

use num_bigint::BigUint;
use serde::Deserialize;

use crate::{json_parser::PublicInput, proof_params::ProofParameters, utils::log2_if_power_of_2};

// For now only the recursive and starknet layouts is supported
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    Dex,
    Plain,
    Recursive,
    RecursiveWithPoseidon,
    Small,
    Starknet,
    StarknetWithKeccak,
}

impl Layout {
    pub fn get_consts(&self) -> LayoutConstants {
        match self {
            Layout::Dex => LayoutConstants::dex(),
            Layout::Plain => LayoutConstants::plain(),
            Layout::Recursive => LayoutConstants::recursive(),
            Layout::RecursiveWithPoseidon => LayoutConstants::recursive_with_poseidon(),
            Layout::Small => LayoutConstants::small(),
            Layout::Starknet => LayoutConstants::starknet(),
            Layout::StarknetWithKeccak => LayoutConstants::starknet_with_keccak(),
        }
    }
    pub fn get_dynamics_or_consts(
        &self,
        dynamic_params: &Option<BTreeMap<String, BigUint>>,
    ) -> Option<LayoutConstants> {
        let consts = self.get_consts();

        let Some(dynamic_params) = dynamic_params else {
            return Some(consts);
        };

        Some(LayoutConstants {
            cpu_component_step: dynamic_params
                .get("cpu_component_step")
                .map(<&BigUint>::try_into)
                .map(Result::ok)?
                .unwrap_or(consts.cpu_component_step),
            constraint_degree: dynamic_params
                .get("constraint_degree")
                .map(<&BigUint>::try_into)
                .map(Result::ok)?
                .unwrap_or(consts.constraint_degree),
            num_columns_first: dynamic_params
                .get("num_columns_first")
                .map(<&BigUint>::try_into)
                .map(Result::ok)?
                .unwrap_or(consts.num_columns_first),
            num_columns_second: dynamic_params
                .get("num_columns_second")
                .map(<&BigUint>::try_into)
                .map(Result::ok)?
                .unwrap_or(consts.num_columns_second),
        })
    }
    pub fn bytes_encode(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layout::Dex => write!(f, "dex"),
            Layout::Plain => write!(f, "plain"),
            Layout::Recursive => write!(f, "recursive"),
            Layout::RecursiveWithPoseidon => write!(f, "recursive_with_poseidon"),
            Layout::Small => write!(f, "small"),
            Layout::Starknet => write!(f, "starknet"),
            Layout::StarknetWithKeccak => write!(f, "starknet_with_keccak"),
        }
    }
}

pub(crate) struct LayoutConstants {
    pub cpu_component_step: u32,
    pub constraint_degree: u32,
    pub num_columns_first: u32,
    pub num_columns_second: u32,
}

impl LayoutConstants {
    pub fn recursive() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            cpu_component_step: 1,
            num_columns_first: 7,
            num_columns_second: 3,
        }
    }
    pub fn starknet() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            cpu_component_step: 1,
            num_columns_first: 9,
            num_columns_second: 1,
        }
    }
    pub fn small() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            cpu_component_step: 1,
            num_columns_first: 23,
            num_columns_second: 2,
        }
    }
    pub fn recursive_with_poseidon() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            cpu_component_step: 1,
            num_columns_first: 6,
            num_columns_second: 2,
        }
    }
    pub fn plain() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            cpu_component_step: 1,
            num_columns_first: 6,
            num_columns_second: 2,
        }
    }
    pub fn starknet_with_keccak() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            cpu_component_step: 1,
            num_columns_first: 12,
            num_columns_second: 3,
        }
    }
    pub fn dex() -> Self {
        LayoutConstants {
            constraint_degree: 2,
            cpu_component_step: 1,
            num_columns_first: 21,
            num_columns_second: 1,
        }
    }
}

impl Layout {
    // https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/air/cpu/board/cpu_air_definition4.inl#L1775-L1776
    pub fn mask_len(&self) -> usize {
        match self {
            Layout::Recursive => 133,
            _ => unimplemented!(),
        }
    }
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/stark/committed_trace.cc#L212-L213
pub fn data_queries_len(public_input: &PublicInput) -> usize {
    let trace_len = trace_len(public_input.layout, public_input.n_steps);
    match public_input.layout {
        Layout::Recursive => 2,
        _ => unimplemented!(),
    }
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/air/cpu/board/cpu_air_definition4.h#L396-L397
pub fn trace_len(layout: Layout, n_steps: u32) -> u32 {
    match layout {
        Layout::Recursive => n_steps * 16 * 1,
        _ => unimplemented!(),
    }
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/stark/stark.cc#L164-L167
pub fn list_of_cosets(public_input: &PublicInput, proof_params: &ProofParameters) -> usize {
    1usize << proof_params.stark.log_n_cosets
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/stark/stark.cc#L204-L205
pub fn fft_bases(layout: Layout, n_steps: u32, proof_params: &ProofParameters) -> u32 {
    let trace_len = trace_len(layout, n_steps);
    let log_trace_len = log2_if_power_of_2(trace_len).unwrap();
    log_trace_len + proof_params.stark.log_n_cosets
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/fri/fri_verifier.cc#L104-L105
pub fn first_fri_layer(proof_params: &ProofParameters) -> u32 {
    proof_params.stark.fri.fri_step_list[0]
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/fri/fri_verifier.cc#L156-L157
pub fn inner_fri_layers(proof_params: &ProofParameters) -> Vec<(u32, u32)> {
    proof_params
        .stark
        .fri
        .fri_step_list
        .iter()
        .skip(1)
        .map(|&x| (x, 1 << x))
        .collect()
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/stark/stark.cc#L303-L304
pub fn fri_degree_bound(proof_params: &ProofParameters) -> u32 {
    let mut expected = proof_params.stark.fri.last_layer_degree_bound;
    for s in &proof_params.stark.fri.fri_step_list {
        expected *= 1 << s
    }
    expected
}

pub fn leaves(proof_params: &ProofParameters) -> Vec<usize> {
    proof_params
        .stark
        .fri
        .fri_step_list
        .iter()
        .skip(1)
        .map(|&x| (1u32 << x + 4) - 16)
        .map(|x| x as usize)
        .collect()
}

// https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/commitment_scheme/packaging_commitment_scheme.cc#L144-L146
pub fn authentications() -> usize {
    256 + 1
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProofStructure {
    pub first_layer_queries: usize,
    pub layer: Vec<usize>,
    pub layer_count: usize,
    pub composition_decommitment: usize,
    pub oods: usize,
    pub last_layer_degree_bound: usize,
    pub authentications: usize,
}

impl ProofStructure {
    pub fn new(proof_params: &ProofParameters, layout: Layout, _n_steps: u32) -> Self {
        let n_queries = proof_params.stark.fri.n_queries;
        let mask_len = layout.mask_len();
        let layout = layout.get_consts();

        ProofStructure {
            // https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/stark/stark.cc#L276-L277
            first_layer_queries: (n_queries * layout.num_columns_first) as usize,

            layer: leaves(proof_params),
            layer_count: (proof_params.stark.fri.fri_step_list.len() - 1).into(),
            composition_decommitment: (n_queries * layout.num_columns_second) as usize,

            // https://github.com/cartridge-gg/stone-prover/blob/fd78b4db8d6a037aa467b7558ac8930c10e48dc1/src/starkware/stark/oods.cc#L92-L93
            oods: mask_len + layout.num_columns_second as usize - 1,
            last_layer_degree_bound: proof_params.stark.fri.last_layer_degree_bound as usize,

            authentications: authentications(),
        }
    }
}

#[test]
fn test_lens() {
    let n_steps = 16384;
    let layout = Layout::Recursive;
    let proof_params = ProofParameters {
        stark: Stark {
            fri: Fri {
                fri_step_list: vec![0, 4, 4, 3],
                last_layer_degree_bound: 128,
                n_queries: 16,
                proof_of_work_bits: 30,
            },
            log_n_cosets: 3,
        },
        n_verifier_friendly_commitment_layers: 0,
    };

    assert_eq!(fri_degree_bound(&proof_params), 262144);

    let result = ProofStructure::new(&proof_params, layout, n_steps);

    let expected = ProofStructure {
        first_layer_queries: 112,
        layer: vec![240, 240, 112],
        layer_count: 3,
        composition_decommitment: 48,
        oods: 135,
        last_layer_degree_bound: 128,
        authentications: 257,
    };
    assert_eq!(result, expected);
}

// expected_fri_degree_bound: 262144
// oracle_degree_bound: 262144
// Traces log size: 3
// Split mask 0: 7
// Answer Queries: 7
// Decommit table: 7
// ElementsToBeTransmitted16x7 87968x0 87968x1 87968x2 87968x3 87968x4 87968x5 87968x6 144819x0 144819x1 144819x2 144819x3 144819x4 144819x5 144819x6 306054x0 306054x1 306054x2 306054x3 306054x4 306054x5 306054x6 337244x0 337244x1 337244x2 337244x3 337244x4 337244x5 337244x6 688551x0 688551x1 688551x2 688551x3 688551x4 688551x5 688551x6 815541x0 815541x1 815541x2 815541x3 815541x4 815541x5 815541x6 969836x0 969836x1 969836x2 969836x3 969836x4 969836x5 969836x6 1017706x0 1017706x1 1017706x2 1017706x3 1017706x4 1017706x5 1017706x6 1089680x0 1089680x1 1089680x2 1089680x3 1089680x4 1089680x5 1089680x6 1097657x0 1097657x1 1097657x2 1097657x3 1097657x4 1097657x5 1097657x6 1233690x0 1233690x1 1233690x2 1233690x3 1233690x4 1233690x5 1233690x6 1321706x0 1321706x1 1321706x2 1321706x3 1321706x4 1321706x5 1321706x6 1510589x0 1510589x1 1510589x2 1510589x3 1510589x4 1510589x5 1510589x6 1734814x0 1734814x1 1734814x2 1734814x3 1734814x4 1734814x5 1734814x6 1820876x0 1820876x1 1820876x2 1820876x3 1820876x4 1820876x5 1820876x6 2048191x0 2048191x1 2048191x2 2048191x3 2048191x4 2048191x5 2048191x6
// Send table: 112
// Traces log size: 3
// Split mask 1: 3
// Answer Queries: 3
// Decommit table: 3
// ElementsToBeTransmitted16x3 87968x0 87968x1 87968x2 144819x0 144819x1 144819x2 306054x0 306054x1 306054x2 337244x0 337244x1 337244x2 688551x0 688551x1 688551x2 815541x0 815541x1 815541x2 969836x0 969836x1 969836x2 1017706x0 1017706x1 1017706x2 1089680x0 1089680x1 1089680x2 1097657x0 1097657x1 1097657x2 1233690x0 1233690x1 1233690x2 1321706x0 1321706x1 1321706x2 1510589x0 1510589x1 1510589x2 1734814x0 1734814x1 1734814x2 1820876x0 1820876x1 1820876x2 2048191x0 2048191x1 2048191x2
// Send table: 48
// Traces log size: 3
// Split mask 2: 2
// Answer Queries: 2
// Decommit table: 2
// ElementsToBeTransmitted16x2 87968x0 87968x1 144819x0 144819x1 306054x0 306054x1 337244x0 337244x1 688551x0 688551x1 815541x0 815541x1 969836x0 969836x1 1017706x0 1017706x1 1089680x0 1089680x1 1097657x0 1097657x1 1233690x0 1233690x1 1321706x0 1321706x1 1510589x0 1510589x1 1734814x0 1734814x1 1820876x0 1820876x1 2048191x0 2048191x1
// Send table: 32
// Decommit table: 16
// ElementsToBeTransmitted16x16 5498x1 5498x2 5498x3 5498x4 5498x5 5498x6 5498x7 5498x8 5498x9 5498x10 5498x11 5498x12 5498x13 5498x14 5498x15 9051x0 9051x1 9051x2 9051x4 9051x5 9051x6 9051x7 9051x8 9051x9 9051x10 9051x11 9051x12 9051x13 9051x14 9051x15 19128x0 19128x1 19128x2 19128x3 19128x4 19128x5 19128x7 19128x8 19128x9 19128x10 19128x11 19128x12 19128x13 19128x14 19128x15 21077x0 21077x1 21077x2 21077x3 21077x4 21077x5 21077x6 21077x7 21077x8 21077x9 21077x10 21077x11 21077x13 21077x14 21077x15 43034x0 43034x1 43034x2 43034x3 43034x4 43034x5 43034x6 43034x8 43034x9 43034x10 43034x11 43034x12 43034x13 43034x14 43034x15 50971x0 50971x1 50971x2 50971x3 50971x4 50971x6 50971x7 50971x8 50971x9 50971x10 50971x11 50971x12 50971x13 50971x14 50971x15 60614x0 60614x1 60614x2 60614x3 60614x4 60614x5 60614x6 60614x7 60614x8 60614x9 60614x10 60614x11 60614x13 60614x14 60614x15 63606x0 63606x1 63606x2 63606x3 63606x4 63606x5 63606x6 63606x7 63606x8 63606x9 63606x11 63606x12 63606x13 63606x14 63606x15 68105x1 68105x2 68105x3 68105x4 68105x5 68105x6 68105x7 68105x8 68105x9 68105x10 68105x11 68105x12 68105x13 68105x14 68105x15 68603x0 68603x1 68603x2 68603x3 68603x4 68603x5 68603x6 68603x7 68603x8 68603x10 68603x11 68603x12 68603x13 68603x14 68603x15 77105x0 77105x1 77105x2 77105x3 77105x4 77105x5 77105x6 77105x7 77105x8 77105x9 77105x11 77105x12 77105x13 77105x14 77105x15 82606x0 82606x1 82606x2 82606x3 82606x4 82606x5 82606x6 82606x7 82606x8 82606x9 82606x11 82606x12 82606x13 82606x14 82606x15 94411x0 94411x1 94411x2 94411x3 94411x4 94411x5 94411x6 94411x7 94411x8 94411x9 94411x10 94411x11 94411x12 94411x14 94411x15 108425x0 108425x1 108425x2 108425x3 108425x4 108425x5 108425x6 108425x7 108425x8 108425x9 108425x10 108425x11 108425x12 108425x13 108425x15 113804x0 113804x1 113804x2 113804x3 113804x4 113804x5 113804x6 113804x7 113804x8 113804x9 113804x10 113804x11 113804x13 113804x14 113804x15 128011x0 128011x1 128011x2 128011x3 128011x4 128011x5 128011x6 128011x7 128011x8 128011x9 128011x10 128011x11 128011x12 128011x13 128011x14
// Send table: 240
// Decommit table: 16
// ElementsToBeTransmitted16x16 343x0 343x1 343x2 343x3 343x4 343x5 343x6 343x7 343x8 343x9 343x11 343x12 343x13 343x14 343x15 565x0 565x1 565x2 565x3 565x4 565x5 565x6 565x7 565x8 565x9 565x10 565x12 565x13 565x14 565x15 1195x0 1195x1 1195x2 1195x3 1195x4 1195x5 1195x6 1195x7 1195x9 1195x10 1195x11 1195x12 1195x13 1195x14 1195x15 1317x0 1317x1 1317x2 1317x3 1317x4 1317x6 1317x7 1317x8 1317x9 1317x10 1317x11 1317x12 1317x13 1317x14 1317x15 2689x0 2689x1 2689x2 2689x3 2689x4 2689x5 2689x6 2689x7 2689x8 2689x9 2689x11 2689x12 2689x13 2689x14 2689x15 3185x0 3185x1 3185x2 3185x3 3185x4 3185x5 3185x6 3185x7 3185x8 3185x9 3185x10 3185x12 3185x13 3185x14 3185x15 3788x0 3788x1 3788x2 3788x3 3788x4 3788x5 3788x7 3788x8 3788x9 3788x10 3788x11 3788x12 3788x13 3788x14 3788x15 3975x0 3975x1 3975x2 3975x3 3975x4 3975x5 3975x7 3975x8 3975x9 3975x10 3975x11 3975x12 3975x13 3975x14 3975x15 4256x0 4256x1 4256x2 4256x3 4256x4 4256x5 4256x6 4256x7 4256x8 4256x10 4256x11 4256x12 4256x13 4256x14 4256x15 4287x0 4287x1 4287x2 4287x3 4287x4 4287x5 4287x6 4287x7 4287x8 4287x9 4287x10 4287x12 4287x13 4287x14 4287x15 4819x0 4819x2 4819x3 4819x4 4819x5 4819x6 4819x7 4819x8 4819x9 4819x10 4819x11 4819x12 4819x13 4819x14 4819x15 5162x0 5162x1 5162x2 5162x3 5162x4 5162x5 5162x6 5162x7 5162x8 5162x9 5162x10 5162x11 5162x12 5162x13 5162x15 5900x0 5900x1 5900x2 5900x3 5900x4 5900x5 5900x6 5900x7 5900x8 5900x9 5900x10 5900x12 5900x13 5900x14 5900x15 6776x0 6776x1 6776x2 6776x3 6776x4 6776x5 6776x6 6776x7 6776x8 6776x10 6776x11 6776x12 6776x13 6776x14 6776x15 7112x0 7112x1 7112x2 7112x3 7112x4 7112x5 7112x6 7112x7 7112x8 7112x9 7112x10 7112x11 7112x13 7112x14 7112x15 8000x0 8000x1 8000x2 8000x3 8000x4 8000x5 8000x6 8000x7 8000x8 8000x9 8000x10 8000x12 8000x13 8000x14 8000x15
// Send table: 240
// Decommit table: 8
// ElementsToBeTransmitted16x8 42x0 42x1 42x2 42x3 42x4 42x5 42x6 70x0 70x1 70x2 70x3 70x4 70x6 70x7 149x0 149x1 149x2 149x4 149x5 149x6 149x7 164x0 164x1 164x2 164x3 164x4 164x6 164x7 336x0 336x2 336x3 336x4 336x5 336x6 336x7 398x0 398x2 398x3 398x4 398x5 398x6 398x7 473x0 473x1 473x2 473x3 473x5 473x6 473x7 496x0 496x1 496x2 496x3 496x4 496x5 496x6 532x1 532x2 532x3 532x4 532x5 532x6 532x7 535x0 535x1 535x2 535x3 535x4 535x5 535x6 602x0 602x1 602x2 602x4 602x5 602x6 602x7 645x0 645x1 645x3 645x4 645x5 645x6 645x7 737x0 737x1 737x2 737x3 737x5 737x6 737x7 847x1 847x2 847x3 847x4 847x5 847x6 847x7 889x1 889x2 889x3 889x4 889x5 889x6 889x7 1000x1 1000x2 1000x3 1000x4 1000x5 1000x6 1000x7
// Send table: 112
