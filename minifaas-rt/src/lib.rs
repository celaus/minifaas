pub mod languages;
pub mod traits;

use crate::languages::Runtime;
use crossbeam_channel::{unbounded as channel, Receiver, Sender};
use hashbrown::{HashMap, HashSet};
use languages::JavaScript;
use minifaas_common::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct RuntimeConfiguration {
    num_threads: usize,
}

impl RuntimeConfiguration {
    pub fn new(num_threads: usize) -> Self {
        RuntimeConfiguration { num_threads }
    }
}

fn execute_function(
    inputs: FunctionInputs,
    response_channel: Sender<RuntimeResponse>,
    runtime: Arc<Runtime>,
    func: Arc<Box<FunctionCode>>,
) {
    let outputs = match func.language {
        ProgrammingLanguage::JavaScript => {
            match runtime.javascript(&func, inputs) {
                Ok(r) => RuntimeResponse::FunctionResponse(r),
                Err(e) => RuntimeResponse::from(e)
            }
        }
        _ => RuntimeResponse::FunctionRuntimeUnavailable(func.language.clone()),
    };
    // the channel could be closed on the other side. we don't care though
    let _ = response_channel.send(outputs);
    drop(response_channel);
}

pub fn create_runtime(
    config: RuntimeConfiguration,
) -> Sender<RuntimeRequest> {
    let (input_channel_sender, input_channel_receiver) = channel::<RuntimeRequest>();

    let timeout = Duration::from_secs(1);
    thread::spawn(move || {
        let mut stop = false;

        let worker_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(config.num_threads)
            .build()
            .unwrap();

        let runtime = Arc::new(Runtime::new());

        while !stop {
            if let Ok(pkg) = input_channel_receiver.recv_timeout(timeout) {
                match pkg {
                    RuntimeRequest::Shutdown => stop = true,
                    RuntimeRequest::FunctionCall(user_code, inputs, tx) => {
                        let _ = worker_pool.install(|| {
                            execute_function(inputs, tx.clone(), runtime.clone(), user_code.clone())
                        });
                    }
                    _ => {}
                }
            }
        }
    });
    input_channel_sender
}
