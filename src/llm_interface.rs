use crate::error::LLMError;
use llm_chain::{prompt, traits::Executor, Parameters};
use llm_chain_llama::Executor as LlamaExecutor;
use llm_chain_llama::{PerExecutor, PerInvocation};

pub struct LLMInterface<T: Executor> {
    pub exec: T,
}
impl LLMInterface<LlamaExecutor> {
    // Create a new local LLM instance with the given parameters
    pub fn new_local_llm(
        model_path: &str,     // Path to the model
        num_threads: u16,     // Number of threads to use
        temp: f32,            // Temperature for sampling
        freq_penalty: f32,    // Frequency penalty for sampling
        output_tokens: usize, // Number of tokens to predict
    ) -> Result<Self, LLMError> {
        // Setup all options
        let exec_options = PerExecutor::new().with_model_path(model_path);
        let mut inv_options = PerInvocation::new();
        inv_options.n_threads = Some(num_threads as i32);
        inv_options.temp = Some(temp);
        inv_options.repeat_penalty = Some(freq_penalty);
        inv_options.n_tok_predict = Some(output_tokens);

        let executor = LlamaExecutor::new_with_options(Some(exec_options), Some(inv_options))
            .map_err(|_| LLMError::InitializingLLMFailed);

        // Looks like the error might not be propagating to here?
        if let Err(e) = executor {
            println!("Failed to initialize LLM interface: {}", e);
            std::process::exit(1);
        }

        Ok(Self { exec: executor? })
    }

    // Submit a prompt to the LLM if it isn't currently busy
    pub async fn submit_prompt(&mut self, prompt_text: &str) -> Result<String, LLMError> {
        println!("Prompt received: {}", prompt_text);
        // Run prompt
        let params = Parameters::new();
        let res = prompt!(prompt_text)
            .run(&params, &self.exec)
            .await
            .map_err(|_| LLMError::SubmittingPromptFailed)?;
        // Acquire result string
        let res_string = res.to_string();

        // Return string
        return Ok(res_string);
    }

    // // Generate embeddings for the given input
    // pub async fn generate_embeddings(&mut self, input_text: &str) -> Result<Vec<i32>, LLMError> {
    //     println!("Generating embeddings for: {}", input_text);
    //     // Run prompt
    //     let res = self.exec.generate_embeddings(input_text);

    //     println!("Embedding Vector:");
    //     for element in &res {
    //         println!("{}", element);
    //     }

    //     // Return string
    //     return Ok(res);
    // }
}
