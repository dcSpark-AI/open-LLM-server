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

pub const APP_VERSION: &str = "0.1.0";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli_interface(); // Get the command line interface arguments
    match matches.subcommand() {
        Some(("run", sub_m)) => return handle_run_command(sub_m).await, // If the subcommand is "run" then call the handle_run_command function
        Some(("help", _)) => println!(""),
        _ => println!("Open LLM Server\nInvalid Command"), // Otherwise print an invalid command message
    }
    Ok(()) // Return Ok if no errors occur
}

// Handle input parsing and starting webserver
async fn handle_run_command(sub_m: &clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    let default_port = 8080;
    let default_threads = 8;
    let default_temp = 0.7;
    let default_freq_penalty = 1.2;
    let default_output_tokens = 2048;

    let port = sub_m
        .value_of("port")
        .unwrap_or(&default_port.to_string())
        .parse::<u16>()
        .unwrap_or(default_port);
    let num_threads = sub_m
        .value_of("num_threads")
        .unwrap_or(&default_threads.to_string())
        .parse::<u16>()
        .unwrap_or(default_threads);
    let temp = sub_m
        .value_of("temp")
        .unwrap_or(&default_temp.to_string())
        .parse::<f32>()
        .unwrap_or(default_temp);
    let freq_penalty = sub_m
        .value_of("freq_penalty")
        .unwrap_or(&default_freq_penalty.to_string())
        .parse::<f32>()
        .unwrap_or(default_freq_penalty);
    let output_tokens = sub_m
        .value_of("output_tokens")
        .unwrap_or(&default_output_tokens.to_string())
        .parse::<usize>()
        .unwrap_or(default_output_tokens);
    let m_arg = sub_m.value_of("model");
    let model_path = match m_arg {
        Some(m) => m.to_string(),
        None => find_local_model().unwrap_or(("model.bin").to_string()),
    };

    model_file_close_check(&model_path);
    return run_webserver(
        &model_path,
        port,
        num_threads,
        temp,
        freq_penalty,
        output_tokens,
    )
    .await;
}

// Intializes the LLM model interface, and starts the web server
async fn run_webserver(
    model_path: &str,
    port: u16,
    num_threads: u16,
    temp: f32,
    freq_penalty: f32,
    output_tokens: usize,
) -> Result<(), Box<dyn Error>> {
    // Setup LLMInterface using an Arc and Mutex to enable sharing the LLM interface across endpoints
    let llm = Arc::new(Mutex::new(LLMInterface::new_local_llm(
        model_path,
        num_threads,
        temp,
        freq_penalty,
        output_tokens,
    )?));

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
