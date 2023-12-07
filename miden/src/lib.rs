#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

// EXPORTS
// ================================================================================================
use crate::utils::collections::Vec;
use assembly::utils::string::ToString;
use crate::utils::string::String;
pub use assembly::{
    ast::{ModuleAst, ProgramAst},
    Assembler, AssemblyError, ParsingError,
};
use vm_core::Felt;
use serde::{Deserialize, Serialize};

pub use processor::{
    crypto, execute, execute_iter, utils, AdviceInputs, AdviceProvider, AsmOpInfo, DefaultHost,
    ExecutionError, ExecutionTrace, Host, Kernel, MemAdviceProvider, Operation, Program,
    ProgramInfo, StackInputs, VmState, VmStateIterator, ZERO,
};
pub use prover::{
    math, prove, Digest, ExecutionProof, FieldExtension, HashFunction, InputError, ProvingOptions,
    StackOutputs, StarkProof, Word,
};
pub use verifier::{verify, VerificationError};

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::console_log;
extern crate console_error_panic_hook;

#[derive(Debug, Serialize, Deserialize)]
pub struct NormalInput {
    pub stack_inputs: StackInputs,
    pub host: DefaultHost<MemAdviceProvider>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VMResult {
    pub outputs: StackOutputsString,
    pub starkproof: ExecutionProof,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StackOutputsString {
    /// The elements on the stack at the end of execution.
    pub stack: Vec<String>,
    /// The overflow table row addresses required to reconstruct the final state of the table.
    pub overflow_addrs: Vec<String>,
}

#[wasm_bindgen]
pub fn execute_zk_program(program_code: String, stack_init: String, advice_tape: String) -> String {
    let options = ProvingOptions::default();

    let assembler = Assembler::default().with_library(&stdlib::StdLibrary::default()).unwrap();

    let program = assembler.compile(&program_code).unwrap();

    let inputs: NormalInput = convert_stackinputs(stack_init, advice_tape);

    let res = prove(&program, inputs.stack_inputs, inputs.host, options);

    assert!(res.is_ok(), "The proof generation fails: {:?}", res);

    let (outputs, proof) = res.unwrap();

    let stack_string: Vec<String> = outputs.stack.iter().map(|(v)| (v.to_string())).collect();

    let overflow_addrs_string: Vec<String> =
        outputs.overflow_addrs.iter().map(|(v)| (v.to_string())).collect();

    let outputs_string = StackOutputsString {
        stack: stack_string,
        overflow_addrs: overflow_addrs_string,
    };

    let result = VMResult {
        outputs: outputs_string,
        starkproof: proof,
    };

    let final_result: String = serde_json::to_string(&result).unwrap();
    return final_result;
}

#[wasm_bindgen]
pub fn generate_program_hash(program_in_assembly: String) -> String {
    let assembler = Assembler::default().with_library(&stdlib::StdLibrary::default()).unwrap();
    let program = assembler.compile(&program_in_assembly).unwrap();
    use vm_core::utils::Serializable;
    let program_hash = program.hash().to_bytes();
    let ph = hex::encode(program_hash);
    return ph;
}

pub fn convert_stackinputs(stack_init: String, advice_tape: String) -> NormalInput {
    let mut stack_inita = Vec::new();
    let mut advice_tapea = Vec::new();
    if stack_init.len() != 0 {
        let stack_init: Vec<&str> = stack_init.split(',').collect();
        stack_inita = stack_init
            .iter()
            .map(|stack_init| Felt::new(stack_init.parse::<u64>().unwrap()))
            .collect();
    };

    if advice_tape.len() != 0 {
        let advice_tape: Vec<&str> = advice_tape.split(',').collect();
        advice_tapea = advice_tape
            .iter()
            .map(|advice_tape| advice_tape.parse::<u64>().unwrap())
            .collect();
    };

    let stack_input: StackInputs = StackInputs::new(stack_inita);

    let advice_inputs = AdviceInputs::default().with_stack_values(advice_tapea).unwrap();

    let mem_advice_provider: MemAdviceProvider = MemAdviceProvider::from(advice_inputs);
    let host: DefaultHost<MemAdviceProvider> = DefaultHost::new(mem_advice_provider);
    let inputs = NormalInput {
        stack_inputs: stack_input,
        host: host
    };

    return inputs;
}


#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}