mod cli;
mod endpoints;
mod error;
mod fs_reading;
mod llm_interface;

use cli::cli_interface;
use endpoints::route_requests;
use fs_reading::{find_local_model, model_file_close_check};
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use llm_interface::LLMInterface;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

// Gets the app version from Cargo.toml
pub const APP_VERSION: &str = "0.1.0";

#[tokio::main]

async fn main() -> Result<(), Box<dyn Error>> {
    let default_port = 9123;

    let matches = cli_interface();
    match matches.subcommand() {
        Some(("run", sub_m)) => {
            let port = sub_m
                .value_of("port")
                .unwrap_or(&default_port.to_string())
                .parse::<u16>()
                .unwrap_or(default_port);
            let m_arg = sub_m.value_of("model");
            let model_path = match m_arg {
                Some(m) => m.to_string(),
                None => find_local_model().unwrap_or(("model.bin").to_string()),
            };
            model_file_close_check(&model_path);
            return run_webserver(&model_path, port).await;
        }
        Some(("help", _)) => println!("Help message"),
        _ => println!("Invalid command"),
    }
    Ok(())
}

// Intializes the LLM model interface, and starts the web server
async fn run_webserver(model_path: &str, port: u16) -> Result<(), Box<dyn Error>> {
    // Setup LLMInterface using an Arc and Mutex to enable sharing the LLM interface across endpoints
    let llm = Arc::new(Mutex::new(LLMInterface::new_local_llm(model_path)?));

    // Setup the endpoints
    let make_svc = make_service_fn(|_conn| {
        let llm = Arc::clone(&llm);
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| route_requests(req, Arc::clone(&llm))))
        }
    });

    // Start the server
    let addr = ([127, 0, 0, 1], port).into();
    let server = Server::bind(&addr).serve(make_svc);
    println!("Server is running on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
