use std::convert::TryInto;
use std::time::Instant;

use cairo_platinum_prover::air::{generate_cairo_proof, PublicInputs};
use cairo_platinum_prover::runner::run::generate_prover_args_from_trace;
use lambdaworks_math::field::fields::fft_friendly::stark_252_prime_field::Stark252PrimeField;
use lambdaworks_math::traits::AsBytes;
use num_bigint::BigUint;
use stark_platinum_prover::proof::options::ProofOptions;
use stark_platinum_prover::proof::stark::StarkProof;
use stark_platinum_prover::traits::AIR;

pub fn generate_proof_from_trace(
    trace_bin_path: &str,
    memory_bin_path: &str,
    proof_options: &ProofOptions,
) -> Option<(
    StarkProof<Stark252PrimeField, Stark252PrimeField>,
    PublicInputs,
)> {
    // ## Generating the prover args
    let timer = Instant::now();
    let Ok((main_trace, pub_inputs)) =
        generate_prover_args_from_trace(trace_bin_path, memory_bin_path)
    else {
        eprintln!("Error generating prover args");
        return None;
    };
    println!("  Time spent: {:?} \n", timer.elapsed());

    // ## Prove
    let timer = Instant::now();
    println!("Making proof ...");
    let proof = match generate_cairo_proof(&main_trace, &pub_inputs, proof_options) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Error generating proof: {:?}", err);
            return None;
        }
    };
    println!("  Time spent in proving: {:?} \n", timer.elapsed());

    Some((proof, pub_inputs))
}

pub fn write_proof_compatible_with_stone(
    proof: StarkProof<Stark252PrimeField, Stark252PrimeField>,
    pub_inputs: PublicInputs,
    proof_path: String,
    options: &ProofOptions,
) {
    let bytes = StoneCompatibleSerializer::serialize_proof::<cairo_platinum_prover::air::CairoAIR>(
        &proof,
        &pub_inputs,
        options,
    );

    let Ok(()) = std::fs::write(&proof_path, format! {"{:#?}", bytes}) else {
        eprintln!("Error writing proof to file: {}", &proof_path);
        return;
    };

    println!("Stone compatible proof written to {}", &proof_path);
}
pub enum Entry<const RADIX: usize> {
    Value(BigUint),
    Array(Vec<BigUint>),
}

impl<const RADIX: usize> std::fmt::Debug for Entry<RADIX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(arg0) => f
                .debug_tuple("Value")
                .field(&arg0.to_str_radix(RADIX.try_into().unwrap()))
                .finish(),
            Self::Array(arg0) => f
                .debug_tuple("Array")
                .field(
                    &arg0
                        .iter()
                        .map(|v| v.to_str_radix(RADIX.try_into().unwrap()))
                        .collect::<Vec<_>>(),
                )
                .finish(),
        }
    }
}

#[derive(Debug, Default)]
pub struct RadixHelper<const RADIX: usize> {
    pub exprs: Vec<Entry<RADIX>>,
}

pub struct ArrayHelper<'a, const RADIX: usize> {
    helper: &'a mut RadixHelper<RADIX>,
    arr: Vec<BigUint>,
}

impl<const RADIX: usize> RadixHelper<RADIX> {
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        let value = BigUint::from_bytes_be(slice);
        self.exprs.push(Entry::Value(value))
    }
    pub fn start_array(&mut self) -> ArrayHelper<RADIX> {
        ArrayHelper {
            helper: self,
            arr: vec![],
        }
    }
    pub fn add_array(&mut self, arr: Vec<BigUint>) {
        self.exprs.push(Entry::Array(arr))
    }
}

impl<'a, const RADIX: usize> ArrayHelper<'a, RADIX> {
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        let value = BigUint::from_bytes_be(slice);
        self.arr.push(value)
    }
    pub fn end_array(self) {
        self.helper.add_array(self.arr)
    }
}

pub struct StoneCompatibleSerializer;

// Used to check the values against main_proof.json or python-generated proof.txt
// The json uses base 16 encoded numbers while the txt uses base 10.
pub type Helper = RadixHelper<16>;

impl StoneCompatibleSerializer {
    pub fn serialize_proof<A>(
        proof: &StarkProof<Stark252PrimeField, Stark252PrimeField>,
        _public_inputs: &A::PublicInputs,
        _options: &ProofOptions,
    ) -> Helper
    where
        A: AIR<Field = Stark252PrimeField, FieldExtension = Stark252PrimeField>,
        A::PublicInputs: AsBytes,
    {
        let mut output = Helper::default();

        Self::append_trace_commitment(proof, &mut output);
        Self::append_composition_polynomial_commitment(proof, &mut output);
        Self::append_out_of_domain_evaluations(proof, &mut output);
        Self::append_fri_commit_phase_commitments(proof, &mut output);
        Self::append_proof_of_work_nonce(proof, &mut output);

        // TODO: Not implemented
        // let fri_query_indexes = Self::get_fri_query_indexes::<A>(proof, public_inputs, options);
        // Self::append_fri_query_phase_first_layer(proof, &fri_query_indexes, &mut output);
        // Self::append_fri_query_phase_inner_layers(proof, &fri_query_indexes, &mut output);

        output
    }
    fn append_trace_commitment(
        proof: &StarkProof<Stark252PrimeField, Stark252PrimeField>,
        output: &mut Helper,
    ) {
        output.extend_from_slice(&proof.lde_trace_main_merkle_root);

        if let Some(lde_trace_aux_merkle_root) = proof.lde_trace_aux_merkle_root {
            output.extend_from_slice(&lde_trace_aux_merkle_root);
        }
    }
    fn append_composition_polynomial_commitment(
        proof: &StarkProof<Stark252PrimeField, Stark252PrimeField>,
        output: &mut Helper,
    ) {
        output.extend_from_slice(&proof.composition_poly_root);
    }

    /// Appends the bytes of the evaluations of the trace `t_1, ..., t_m` and composition polynomial parts
    /// `H_1, ..., H_s` at the out of domain challenge `z`, its shifts `g^i z` and its power `z^s`, respectively.
    /// These are sorted as follows: first the evaluations of the trace in increasing order of
    /// trace column and shift number. Then all the evaluations of the parts of the composition
    /// polynomial. That is:
    ///
    /// t_1(z), ..., t_1(g^K z), t_2(z), ..., t_2(g^K z), ..., t_m(g z), ..., t_m(g^K z), H_1(z^s), ..., H_s(z^s).
    ///
    /// Here, K is the length of the frame size.
    fn append_out_of_domain_evaluations(
        proof: &StarkProof<Stark252PrimeField, Stark252PrimeField>,
        output: &mut Helper,
    ) {
        for i in 0..proof.trace_ood_evaluations.width {
            let mut arr = output.start_array();
            for j in 0..proof.trace_ood_evaluations.height {
                arr.extend_from_slice(&proof.trace_ood_evaluations.get_row(j)[i].as_bytes());
            }
            arr.end_array()
        }

        let mut arr = output.start_array();
        for elem in proof.composition_poly_parts_ood_evaluation.iter() {
            arr.extend_from_slice(&elem.as_bytes());
        }
        arr.end_array()
    }

    fn append_fri_commit_phase_commitments(
        proof: &StarkProof<Stark252PrimeField, Stark252PrimeField>,
        output: &mut Helper,
    ) {
        let mut arr = output.start_array();

        for p in &proof.fri_layers_merkle_roots {
            arr.extend_from_slice(p)
        }

        arr.end_array();

        output.extend_from_slice(&proof.fri_last_value.as_bytes());
    }
    fn append_proof_of_work_nonce(
        proof: &StarkProof<Stark252PrimeField, Stark252PrimeField>,
        output: &mut Helper,
    ) {
        if let Some(nonce_value) = proof.nonce {
            output.extend_from_slice(&nonce_value.to_be_bytes());
        }
    }
}
