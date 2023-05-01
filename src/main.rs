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

async fn prompt_request(
    req: Request<Body>,
    llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>,
) -> Result<Response<Body>, LLMError> {
    let mut llm_guard = llm.lock().await;
    let content = llm_guard.submit_prompt().await?;

    Ok(Response::new(Body::from(content)))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup LLMInterface using an Arc and Mutex to enable sharing the LLM interface across endpoints
    let llm = Arc::new(Mutex::new(LLMInterface::new_local_llm("model.bin")?));

    // Setup the endpoints
    let prompt_svc = make_service_fn(|_conn| {
        let llm = Arc::clone(&llm);
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| prompt_request(req, Arc::clone(&llm))))
        }
    });

    // Start the server
    let addr = ([127, 0, 0, 1], 8080).into();
    let server = Server::bind(&addr).serve(prompt_svc);
    println!("Server is running on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
