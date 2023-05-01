mod error;
mod llm_interface;

use error::LLMError;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use llm_chain_llama::Executor as LlamaExecutor;
use llm_interface::LLMInterface;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

// Endpoint logic when a prompt is requested
async fn prompt_request(
    req: Request<Body>,
    llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>,
) -> Result<Response<Body>, LLMError> {
    let content = match llm.try_lock() {
        Ok(mut llm_guard) => llm_guard.submit_prompt().await?,
        Err(_) => "LLM Is Busy".to_string(),
    };

    Ok(Response::new(Body::from(content)))
}

// Endpoint logic for checking if LLM is busy
async fn is_busy_request(
    req: Request<Body>,
    llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>,
) -> Result<Response<Body>, LLMError> {
    let is_available = llm.try_lock();
    let content = if is_available.is_ok() {
        "false"
    } else {
        "true"
    };

    Ok(Response::new(Body::from(content)))
}

async fn route_request(
    req: Request<Body>,
    llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>,
) -> Result<Response<Body>, LLMError> {
    match req.uri().path() {
        "/prompt" => prompt_request(req, llm).await,
        "/is_busy" => is_busy_request(req, llm).await,
        _ => {
            let mut res = Response::new(Body::empty());
            //*res.status_mut() = StatusCode::NOT_FOUND;
            Ok(res)
        }
    }
}

#[tokio::main(flavor = "current_thread")]
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
