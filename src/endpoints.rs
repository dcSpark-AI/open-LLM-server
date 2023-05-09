use crate::error::LLMError;
use crate::llm_interface::LLMInterface;
use crate::APP_VERSION;
use futures::Future;
use hyper::body::to_bytes;
use hyper::header;
use hyper::{Body, Request, Response};
use llm_chain_llama::Executor as LlamaExecutor;
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::Mutex;

// Struct to represent submit prompt input
#[derive(Serialize, Deserialize, Debug)]
struct PromptInput {
    prompt: String,
}

// Struct to represent a submit prompt response
#[derive(Serialize)]
struct PromptResponse {
    success: bool,
    response: String,
}

// Struct to represent the is_busy endpoint response
#[derive(Serialize)]
struct IsBusyResponse {
    success: bool,
    is_busy: bool,
}

impl IsBusyResponse {
    async fn new(llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>, endpoint_success: bool) -> Self {
        // Attempt to acquire the LLM mutex lock
        let is_available = llm.try_lock();
        // Determine whether the lock was acquired and create the response object
        let resp = Self {
            success: endpoint_success,
            is_busy: is_available.is_err(),
        };
        drop(is_available);
        return resp;
    }
}

// Routes requests based on their URI
pub async fn route_requests(
    req: Request<Body>,
    llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>,
) -> Result<Response<Body>, LLMError> {
    // Pre-check if the LLM is busy before doing any other routing
    let response = IsBusyResponse::new(Arc::clone(&llm), false).await;
    if response.is_busy && req.uri().path() != "/is_busy" {
        return is_busy_http_response(response).await;
    }

    // Check if there is an API key/run checks
    let llm_clone = Arc::clone(&llm);
    if let Err(e) = check_api_key(&req, &llm_clone).await {
        let error_msg = format!("{}", e);
        return Ok(Response::builder()
            .status(hyper::StatusCode::UNAUTHORIZED)
            .body(Body::from(error_msg))
            .unwrap());
    }

    // If the LLM isn't busy and API checks pass,
    // match the URI path to the appropriate endpoint function
    // and return the result
    match req.uri().path() {
        // Root endpoint
        "/" => root_endpoint(req).await,
        // Spawn a new task to handle a prompt request and return the result
        "/submit_prompt" => spawn_and_get_result(req, llm, submit_prompt_endpoint).await,
        // Spawn a new task to handle generating embeddings
        // "/generate_embeddings" => {
        //     spawn_and_get_result(req, llm, generate_embeddings_endpoint).await
        // }
        // Handle a prompt request by streaming the result back to the client
        // "/submit_prompt_streaming" => {
        //     spawn_and_get_result(req, llm, submit_prompt_streaming_endpoint).await
        // }
        // Return a response indicating whether the LLM is currently locked.
        // This endpoint is required for setting the success value properly.
        "/is_busy" => is_busy_endpoint(llm).await,
        // Return an empty response for any other path
        _ => Ok(Response::new(Body::empty())),
    }
}

// Verifies that the api key checks pass
async fn check_api_key(
    req: &Request<Body>,
    llm: &Arc<Mutex<LLMInterface<LlamaExecutor>>>,
) -> Result<(), LLMError> {
    // Lock the LLM and check if there is an API key
    let llm_guard = llm.lock().await;
    if let Some(api_key) = &llm_guard.api_key {
        // Check if the request includes an 'Authorization' header
        if let Some(auth_header) = req.headers().get("Authorization") {
            // If the header is not equal to the API key, return an error
            if auth_header != api_key {
                return Err(LLMError::Custom("Invalid API key".into()));
            }
        } else {
            // If no 'Authorization' header is present, return an error
            return Err(LLMError::Custom("No API key provided".into()));
        }
    }
    // Drop the lock
    drop(llm_guard);

    // If we reached this point, the API key is valid or there was no API key to check
    Ok(())
}

// Basic root endpoint
async fn root_endpoint(_req: Request<Body>) -> Result<Response<Body>, LLMError> {
    let response_body = "Open LLM Server v".to_string() + APP_VERSION;
    let response = Response::builder()
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(response_body))
        .map_err(LLMError::from)?;

    Ok(response)
}

// Returns a response indicating whether the LLM is currently locked
// This returns success == true;
async fn is_busy_endpoint(
    llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>,
) -> Result<Response<Body>, LLMError> {
    let response = IsBusyResponse::new(llm, true).await;
    is_busy_http_response(response).await
}

// Takes an IsBusyResponse and builds it into a proper http response
async fn is_busy_http_response(response: IsBusyResponse) -> Result<Response<Body>, LLMError> {
    // Serialize the response object to JSON
    let body = serde_json::to_string(&response)?;
    // Return a new response with the JSON content
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))?)
}

// Handle a prompt request and send the response through a channel
async fn submit_prompt_endpoint(
    mut req: Request<Body>,
    llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>,
    tx: oneshot::Sender<Result<Response<Body>, LLMError>>,
) {
    // Extract the body from the request and convert it to a string
    let body_bytes = to_bytes(req.body_mut()).await.unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Deserialize the body string into a PromptInput struct
    let input: PromptInput = match serde_json::from_str(&body_str) {
        Ok(input) => input,
        Err(_) => {
            if tx
                .send(Err(LLMError::Custom(
                    "Failed to parse request body".to_string(),
                )))
                .is_err()
            {
                eprintln!("Failed to send prompt response.");
            }
            return;
        }
    };

    // Attempt to acquire the LLM mutex lock and submit the prompt
    let content = match llm.try_lock() {
        Ok(mut llm_guard) => llm_guard.submit_prompt(&input.prompt).await,
        // If the LLM is locked, return an error
        Err(_) => Err(LLMError::Custom("LLM Is Busy".to_string())),
    };

    // Create a response based on the result of the prompt request
    let response = match content {
        Ok(content) => PromptResponse {
            success: true,
            response: content,
        },
        Err(error) => PromptResponse {
            success: false,
            response: error.to_string(),
        },
    };

    // Convert the response to JSON
    let body = match serde_json::to_string(&response) {
        Ok(body) => body,
        Err(_) => {
            if tx
                .send(Err(LLMError::Custom(
                    "Failed to convert response to JSON".to_string(),
                )))
                .is_err()
            {
                eprintln!("Failed to send prompt response.");
            }
            return;
        }
    };

    // Create a JSON response
    let res = Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .map_err(LLMError::from);

    // Send the response through the channel
    if tx.send(res).is_err() {
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
    Fut: Future<Output = ()> + 'static,
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

// Handle a prompt request and send the response through a channel
// async fn generate_embeddings_endpoint(
//     mut req: Request<Body>,
//     llm: Arc<Mutex<LLMInterface<LlamaExecutor>>>,
//     tx: oneshot::Sender<Result<Response<Body>, LLMError>>,
// ) {
//     // Extract the body from the request and convert it to a string
//     let body_bytes = to_bytes(req.body_mut()).await.unwrap();
//     let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

//     // Deserialize the body string into a PromptInput struct
//     let input: PromptInput = match serde_json::from_str(&body_str) {
//         Ok(input) => input,
//         Err(_) => {
//             if tx
//                 .send(Err(LLMError::Custom(
//                     "Failed to parse request body".to_string(),
//                 )))
//                 .is_err()
//             {
//                 eprintln!("Failed to send prompt response.");
//             }
//             return;
//         }
//     };

//     // Attempt to acquire the LLM mutex lock and submit the prompt
//     let res = match llm.try_lock() {
//         Ok(mut llm_guard) => llm_guard.generate_embeddings(&input.prompt).await,
//         // If the LLM is locked, return an error
//         Err(_) => Err(LLMError::Custom("LLM Is Busy".to_string())),
//     };

//     // Create a response based on the result of the prompt request
//     let response = match res {
//         Ok(embeddings) => PromptResponse {
//             success: true,
//             response: embeddings[0].to_string(),
//         },
//         Err(error) => PromptResponse {
//             success: false,
//             response: error.to_string(),
//         },
//     };

//     // Convert the response to JSON
//     let body = match serde_json::to_string(&response) {
//         Ok(body) => body,
//         Err(_) => {
//             if tx
//                 .send(Err(LLMError::Custom(
//                     "Failed to convert response to JSON".to_string(),
//                 )))
//                 .is_err()
//             {
//                 eprintln!("Failed to send prompt response.");
//             }
//             return;
//         }
//     };

//     // Create a JSON response
//     let res = Response::builder()
//         .header(header::CONTENT_TYPE, "application/json")
//         .body(Body::from(body))
//         .map_err(LLMError::from);

//     // Send the response through the channel
//     if tx.send(res).is_err() {
//         eprintln!("Failed to send prompt response.");
//     }
// }
