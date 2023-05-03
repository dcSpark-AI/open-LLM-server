use crate::error::LLMError;
use llm_chain::{parameters, prompt, traits::Executor};
use llm_chain_llama::Executor as LlamaExecutor;
use llm_chain_llama::{PerExecutor, PerInvocation};

pub struct LLMInterface<T: Executor> {
    pub exec: T,
}
impl LLMInterface<LlamaExecutor> {
    pub fn new_local_llm(
        model_path: &str,
        num_threads: u16,
        temp: f32,
        freq_penalty: f32,
        output_tokens: usize,
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
        let res = prompt!(prompt_text)
            .run(&parameters!(), &self.exec)
            .await
            .map_err(|_| LLMError::SubmittingPromptFailed)?;
        // Acquire result string
        let res_string = res.to_string();

        // Return string
        return Ok(res_string);
    }
}
