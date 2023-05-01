mod error;
mod llm_interface;

use error::LLMError;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use llm_chain_llama::Executor as LlamaExecutor;
use llm_interface::LLMInterface;
use std::error::Error;
use std::sync::Arc;

async fn handle_request(
    req: Request<Body>,
    llm: Arc<LLMInterface<LlamaExecutor>>,
) -> Result<Response<Body>, LLMError> {
    let mut llm = LLMInterface::with_llama_executor()?;
    let content = llm.submit_prompt().await?;

    Ok(Response::new(Body::from(content)))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup LLMInterface using an Arc to enable sharing the LLM across endpoints
    let llm = Arc::new(LLMInterface::with_llama_executor()?);

    // Setup the actual webserver
    let make_svc = make_service_fn(|_conn| {
        let llm = Arc::clone(&llm);
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| handle_request(req, Arc::clone(&llm))))
        }
    });

    let addr = ([127, 0, 0, 1], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);

    println!("Server is running on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
