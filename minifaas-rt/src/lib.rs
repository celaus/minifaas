pub mod languages;
pub mod traits;
pub mod triggers;
pub mod errors;

use crate::languages::Runtime;
use languages::{FunctionCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;
use errors::LanguageRuntimeError;
use triggers::{FunctionInputs, FunctionOutputs};
use std::sync::Arc;
use std::collections::HashMap;
use languages::JavaScript;

#[derive(Serialize, Deserialize)]
#[serde(tag = "lang")]
enum ProgrammingLanguage {
    JavaScript,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "trigger")]
enum Trigger {
    Http { method: String },
}

#[derive(Serialize, Deserialize)]
struct UserFunctionDeclaration {
    name: String,
    code: String,
    #[serde(flatten)]
    trigger: Trigger,

    #[serde(flatten)]
    language: ProgrammingLanguage,
}


pub enum RuntimeRequest {
    Shutdown,
    FunctionCall(String, FunctionInputs)
}

pub enum RuntimeResponse {
    FunctionResponse(FunctionOutputs),
    FunctionNotFoundResponse(String)
}


pub struct RuntimeConfiguration {
    num_threads: usize,
    functions: Vec<UserFunctionDeclaration>, 

}

fn execute_function(inputs: FunctionInputs, response_channel: Sender<RuntimeResponse>, runtime: Arc<Box<Runtime>>, func: Arc<Box<FunctionCode>>) -> Result<(), LanguageRuntimeError> {
    let outputs = runtime.javascript(func, inputs)?;
    response_channel.send(RuntimeResponse::FunctionResponse(outputs));
    Ok(())
}

pub fn create_runtime(config: RuntimeConfiguration
) -> Result<(Sender<RuntimeRequest>, Receiver<RuntimeResponse>), LanguageRuntimeError> {

    let (input_channel_sender, input_channel_receiver) = channel::<RuntimeRequest>();
    let (output_channel_sender, output_channel_receiver) = channel::<RuntimeResponse>();


    let timeout = Duration::from_secs(1);

    thread::spawn(move || {
        let mut stop = false;

        let worker_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.num_threads)
        .build()
        .unwrap();
        let functions = HashMap::<String, (Arc<Box<FunctionCode>>, Arc<Box<Runtime>>)>::new();
        while !stop {
            if let Ok(pkg) = input_channel_receiver.recv_timeout(timeout) {
                match pkg {
                    RuntimeRequest::Shutdown => stop = true,
                    RuntimeRequest::FunctionCall(name, inputs) => {
                        functions.get(&name).map(|f|{
                            let tx = output_channel_sender.clone();
                            let (func, exec) = f;
                            worker_pool.install(|| execute_function(inputs, tx, exec.clone(), func.clone()));

                        }).or_else(||{
                            output_channel_sender.clone().send(RuntimeResponse::FunctionNotFoundResponse(name));
                            None
                        });
                    },
                    _ => {}
                }
            }
        }
    });
    
    Err(LanguageRuntimeError::None{})
    //let (rx, tx) = channel<RuntimeResponse>();
}
