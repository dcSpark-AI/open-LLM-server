use crate::error::LLMError;
use crate::llm_interface::LLMInterface;
use futures::Future;
use hyper::{Body, Request, Response};
use llm_chain_llama::Executor as LlamaExecutor;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::Mutex;

// Routes requests based on their URI
pub async fn route_requests(
    req: Request<Body>,
    llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>,
) -> Result<Response<Body>, LLMError> {
    // Match the URI path to the appropriate endpoint function
    match req.uri().path() {
        // Spawn a new task to handle a prompt request and return the result
        "/prompt" => spawn_and_get_result(req, llm, prompt_endpoint).await,
        // Return a response indicating whether the LLM is currently locked
        "/is_busy" => is_busy_endpoint(llm).await,
        // Return an empty response for any other path
        _ => Ok(Response::new(Body::empty())),
    }
}

// Returns a response indicating whether the LLM is currently locked
async fn is_busy_endpoint(
    llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>,
) -> Result<Response<Body>, LLMError> {
    // Attempt to acquire the LLM mutex lock
    let is_available = llm.try_lock();
    // Determine whether the lock was acquired and set the response content accordingly
    let content = if is_available.is_ok() {
        "false"
    } else {
        "true"
    };
    // Return a new response with the content
    Ok(Response::new(Body::from(content)))
}

// Handle a prompt request and send the response through a channel
async fn prompt_endpoint(
    req: Request<Body>,
    llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>,
    tx: oneshot::Sender<Result<Response<Body>, LLMError>>,
) {
    // Attempt to acquire the LLM mutex lock and submit the prompt
    let content = match llm.try_lock() {
        Ok(mut llm_guard) => llm_guard.submit_prompt().await,
        // If the LLM is locked, return an error
        Err(_) => Err(LLMError::Custom("LLM Is Busy".to_string())),
    };
    // Create a response based on the result of the prompt request
    let res = match content {
        Ok(content) => Response::new(Body::from(content)),
        Err(error) => Response::new(Body::from("Failed submitting prompt request to LLM")),
    };
    // Send the response through the channel
    if tx.send(Ok(res)).is_err() {
        eprintln!("Failed to send prompt response.");
    }
}
// Spawns a new task to handle a request and returns the result
async fn spawn_and_get_result<F, Fut>(
    req: Request<Body>,
    llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>,
    func: F,
) -> Result<Response<Body>, LLMError>
where
    // Define the function and future types
    F: Fn(
            Request<Body>,
            Arc<Mutex<LLMInterface<LlamaExecutor>>>,
            oneshot::Sender<Result<Response<Body>, LLMError>>,
        ) -> Fut
        + Send
        + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    // Create a new channel to receive the response
    let (tx, rx) = oneshot::channel();
    // Spawn a new task to handle the request
    tokio::task::spawn(async move {
        // Use `block_in_place` to run the blocking operation on the current thread
        // and `block_on` to wait for the future to complete.
        // (In practice the LLM will spawn new threads anyways).
        tokio::task::block_in_place(|| futures::executor::block_on(func(req, llm, tx)));
    });
    // Await the response from the channel or return an error if it fails
    rx.await
        .unwrap_or_else(|_| Err(LLMError::Custom("Failed to get response.".to_string())))
}
