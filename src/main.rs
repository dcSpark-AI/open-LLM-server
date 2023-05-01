mod llm_interface;

use std::error::Error;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use llm_chain::{executor, parameters, prompt};
use llm_interface::{LLMInterface};



async fn call_llama() -> Result<String, Box<dyn std::error::Error>> {
    // Init the LLMInterface struct
    let mut llm = LLMInterface::with_llama_executor()?;
    let res = llm.submit_prompt().await?;
    
    println!("Llama: {}", res);
    Ok(res.to_string())
}

async fn call_gpt3() -> Result<String, Box<dyn std::error::Error>> {
    let exec = executor!()?;
    // Create our prompt
    let res = prompt!(
        "You are a robot assistant for making personalized greetings",
        "Make a personalized greeting for Joe"
    )
    .run(&parameters!(), &exec) // ...and run it
    .await?;
    println!("OpenAI: {}", res);

    Ok(res.to_string())
}

async fn handle_request(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let content = match call_llama().await {
        Ok(result) => result,
        Err(error) => format!("Error calling GPT-3: {}", error),
    };
    Ok(Response::new(Body::from(content)))
}


#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup LLMInterface
    let llm = LLMInterface::with_llama_executor()?;
    // Create a channel to provide interaction between llm interface and the webserver requests
    // ...


    // Setup the actual webserver (and TODO: feed channel to request handlers)
    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(handle_request)) });

    let addr = ([127, 0, 0, 1], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);

    println!("Server is running on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}