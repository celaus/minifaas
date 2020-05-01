use crate::triggers::{FunctionInputs, FunctionOutputs};
use crate::languages::{FunctionCode, Result, SourceCode, Runtime};
use ducc::{Ducc, Result as DuccResult, Value};
use std::sync::Arc;

pub trait JavaScript {
    fn javascript(&self, func: Arc<Box<FunctionCode>>, inputs: FunctionInputs) -> Result<FunctionOutputs> {
        let code = func.str_source();
        let ducc = Ducc::new();
        let func: Value = ducc.compile(code, None).unwrap().call(()).unwrap();
        let func = if let Value::Function(f) = func {
            f
        } else {
            unreachable!();
        };
        let result: f64 = func.call(()).unwrap();
        println!("Result!! {}", result);
        Ok(FunctionOutputs::None)
    }
}

impl JavaScript for Runtime {}

// pub struct DuccJS {
// }

// impl Compiler for DuccJS {
//     type CompilerCode = CompiledJS;
//     fn compile(&self, code: &str) -> Result<Box<Self::CompilerCode>> {
//         Ok(Box::new(CompiledJS::new(code.to_owned())))
//     }
// }

// impl Executor for DuccJS {
//     type ByteCodeType = CompiledJS;
//     fn run(&self, func: Arc<Box<Self::ByteCodeType>>, inputs: Option<FunctionInputs>) -> Result<FunctionOutputs> {
//         let code = func.executable();
//         let ducc = Ducc::new();
//         let func: Value = ducc.compile(code, None).unwrap().call(()).unwrap();
//         let func = if let Value::Function(f) = func { f } else { unreachable!(); };
//         let result: f64 = func.call(()).unwrap();
//         println!("Result!! {}", result);
//         Ok(FunctionOutputs::None)
//     }
// }

// pub struct CompiledJS {
//     code: String,
// }

// impl CompiledJS {
//     pub fn new(code: String) -> CompiledJS {
//         CompiledJS {
//             code
//         }
//     }
// }
