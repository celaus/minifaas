use crate::languages::{FunctionCode, Result, Runtime, SourceCode};
use minifaas_common::{errors::ExecutionError, FunctionInputs, FunctionOutputs};
use std::collections::HashMap;
use deno_core::Op;
use deno_core::ZeroCopyBuf;
use deno_core::{CoreIsolate, CoreIsolateState, StartupData};
///
/// A JavaScript compiler and runtime based on Duktape via a Rust binding. The trait provides a function `javascript` to the implementing struct which compiles and runs code inside a prepared environment. 
/// The code's required output depends on its input - which in turn depends on the trigger. Http provides headers and body as an input and wants to see body, headers, and status code as return values. Other triggers will have similar requirements and all of them are implemented here. 
/// 
pub trait JavaScript {

    /// 
    /// Compile and run the provided code inside a JavaScript environment. Add inputs to the function and check outputs for required return values. 
    /// 
    fn javascript(&self, func: &FunctionCode, inputs: FunctionInputs) -> Result<FunctionOutputs> {
        let code = func.str_source();

        let mut isolate = CoreIsolate::new(StartupData::None, false);
      
        match isolate.execute("<anon>", code) {
            Ok(return_value) => {
                let result = match inputs {
                    /// HTTP triggered, thus a HTTP return value
                    FunctionInputs::Http { params, headers, body } => {
        
                        let body: String = return_values.get("body")
                            .and_then(|v| String::from_value(v.clone(), &ducc).ok())
                            .unwrap_or("".to_owned());

                        let headers: HashMap<String, Option<String>> = return_values.get("headers")
                            .and_then(|v| HashMap::from_value(v.clone(), &ducc).ok())
                            .unwrap_or_default();

                        let status: u16 = return_values.get("status")
                            .and_then(|v| u16::from_value(v.clone(), &ducc).ok())
                            .unwrap_or(403); // http status code for BadRequest
                                        
                            FunctionOutputs::Http{ headers, body, status_code: status }
                        },
                        _ => FunctionOutputs::None
                    }
                };
                
                Ok(result)
            }
            Err(compiler_error) => {
                let result = match compiler_error.kind {
                    ErrorKind::RuntimeError { code: _, name } => {
                        ExecutionError::CompilerError(name, compiler_error.context)
                    }
                    _ => ExecutionError::GeneralExecutionError(compiler_error.context),
                };
                Err(result)
            }
        }
    }
}

impl JavaScript for Runtime {}
