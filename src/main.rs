mod endpoints;
mod error;
mod llm_interface;

use endpoints::route_requests;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use llm_interface::LLMInterface;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup LLMInterface using an Arc and Mutex to enable sharing the LLM interface across endpoints
    let llm = Arc::new(Mutex::new(LLMInterface::new_local_llm("model.bin")?));

    // Setup the webserver
    let make_svc = make_service_fn(|_conn| {
        let llm = Arc::clone(&llm);
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| route_requests(req, Arc::clone(&llm))))
        }
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
