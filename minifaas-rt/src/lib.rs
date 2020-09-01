pub mod languages;
pub mod traits;

use crate::languages::Runtime;
use crossbeam_channel::{unbounded as channel, Sender};
use languages::JavaScript;
use minifaas_common::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use log::{error, debug, info, trace, warn};


/// Move this crate to actors:
/// Management actor that manages access to the function db? 
/// Runtime actors: per each language one actor? 
/// 

///
/// Configuration for the Function runtime
/// 
pub struct RuntimeConfiguration {
    num_threads: usize,
}

impl RuntimeConfiguration {

    ///
    /// New runtime config
    /// 
    pub fn new(num_threads: usize) -> Self {
        RuntimeConfiguration { num_threads }
    }
}


///
/// Execute a Function in a thread and send the response via the provided `Sender<>`
/// 
fn execute_function(
    inputs: FunctionInputs,
    response_channel: Sender<RuntimeResponse>,
    runtime: Arc<Runtime>,
    func: Arc<Box<FunctionCode>>,
) {
    info!("Executing function {:?} with inputs {:?}", func, inputs);
    let outputs = match func.language {
        ProgrammingLanguage::JavaScript => match runtime.javascript(&func, inputs) {
            Ok(r) => RuntimeResponse::FunctionResponse(r),
            Err(e) => RuntimeResponse::from(e),
        },
        _ => RuntimeResponse::FunctionRuntimeUnavailable(func.language.clone()),
    };
    // the channel could be closed on the other side. we don't care though
    let _ = response_channel.send(outputs);
    drop(response_channel);
}

///
/// Creates a runtime based on the configuration and returns a command channel to invoke things with.
///
pub fn create_runtime(config: RuntimeConfiguration) -> Sender<RuntimeRequest> {
    let (input_channel_sender, input_channel_receiver) = channel::<RuntimeRequest>();

    let timeout = Duration::from_secs(1);
    thread::spawn(move || {
        let mut stop = false;

        let worker_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(config.num_threads)
            .build()
            .expect("Couldn't create runtime threadpool");

        let runtime = Arc::new(Runtime::new());
        info!("Language runtime initialized with {} threads and accepting work", config.num_threads);
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
