mod error;
mod llm_interface;

use error::LLMError;
use futures::executor;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use llm_chain_llama::Executor as LlamaExecutor;
use llm_interface::LLMInterface;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::Mutex;

async fn prompt_request(
    req: Request<Body>,
    llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>,
    tx: mpsc::Sender<Result<Response<Body>, LLMError>>,
) {
    let content = match llm.try_lock() {
        Ok(mut llm_guard) => llm_guard.submit_prompt().await,
        Err(_) => Err(LLMError::Custom("LLM Is Busy".to_string())),
    };

    let res = match content {
        Ok(content) => Response::new(Body::from(content)),
        Err(error) => Response::new(Body::from("Failed submitting prompt request to LLM")),
    };

    if tx.send(Ok(res)).await.is_err() {
        eprintln!("Failed to send prompt response.");
    }
}

async fn route_request(
    req: Request<Body>,
    llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>,
) -> Result<Response<Body>, LLMError> {
    match req.uri().path() {
        "/prompt" => {
            let (tx, mut rx) = mpsc::channel(1);
            tokio::task::spawn(async move {
                let result = tokio::task::block_in_place(|| {
                    futures::executor::block_on(prompt_request(req, llm, tx))
                });
            });
            rx.recv().await.unwrap_or_else(|| {
                Err(LLMError::Custom(
                    "Failed to get prompt response.".to_string(),
                ))
            })
        }
        "/is_busy" => {
            let is_available = llm.try_lock();
            let content = if is_available.is_ok() {
                "false"
            } else {
                "true"
            };

            Ok(Response::new(Body::from(content)))
        }
        _ => Ok(Response::new(Body::empty())),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup LLMInterface using an Arc and Mutex to enable sharing the LLM interface across endpoints
    let llm = Arc::new(Mutex::new(LLMInterface::new_local_llm("model.bin")?));

    // Setup the webserver
    let make_svc = make_service_fn(|_conn| {
        let llm = Arc::clone(&llm);
        async move { Ok::<_, hyper::Error>(service_fn(move |req| route_request(req, Arc::clone(&llm)))) }
    });

    // Start the server
    let addr = ([127, 0, 0, 1], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);
    println!("Server is running on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
