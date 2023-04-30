use axum::{
    routing::get,
    Router,
};
use llm_chain::{executor, parameters, prompt};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup local LLM and issue initial hard-coded prompt
    let exec = executor!(llama)?;
    let res = prompt!("Write a typescript function to reverse a list manually, and then return the list.")
        .run(&parameters!(), &exec)
        .await?;
    println!("{}", res);
        
    // Run on localhost:3000, serve result of the prompt
    let app = Router::new().route("/", get(|| async move { res.as_str().to_string() }));
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
