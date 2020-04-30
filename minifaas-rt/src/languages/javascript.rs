use crate::languages::{Compiler, FunctionInputs, FunctionOutputs, Executor, CompiledFunction};
use ducc::{Ducc, Value, Result as DuccResult};
use crate::languages::Result;

pub struct DuccJS {
}

impl Compiler for DuccJS {
    type CompilerCode = CompiledJS;
    fn compile(&self, code: &str) -> Result<Box<Self::CompilerCode>> {
        Ok(Box::new(CompiledJS::new(code.to_owned())))
    }
}

impl Executor for DuccJS {
    type ByteCodeType = CompiledJS;
    fn run(&self, func: Box<Self::ByteCodeType>, inputs: Option<FunctionInputs>) -> Result<FunctionOutputs> {
        let code = func.executable();
        let ducc = Ducc::new();
        let func: Value = ducc.compile(code, None).unwrap().call(()).unwrap();
        let func = if let Value::Function(f) = func { f } else { unreachable!(); };
        let result: f64 = func.call(()).unwrap();
        println!("Result!! {}", result);
        
        Ok(FunctionOutputs::None)
    }
}



pub struct CompiledJS {
    code: String,
}

impl CompiledJS {
    pub fn new(code: String) -> CompiledJS {
        CompiledJS {
            code
        }
    }
}

impl CompiledFunction for CompiledJS {
    type ByteCodeType = String;
    fn executable(&self) -> &Self::ByteCodeType {
        &self.code
    }
}