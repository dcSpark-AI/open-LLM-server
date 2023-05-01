use crate::error::LLMError;
use llm_chain::{parameters, prompt, traits::Executor};
use llm_chain_llama::Executor as LlamaExecutor;
use llm_chain_openai::chatgpt::Executor as ChatGPTExecutor;

use llm_chain_llama::{PerExecutor, PerInvocation};

pub struct LLMInterface<T: Executor> {
    exec: T,
}
impl LLMInterface<LlamaExecutor> {
    pub fn new_local_llm(model_path: &str) -> Result<Self, LLMError> {
        let exec_options = PerExecutor::new().with_model_path(model_path);
        let mut inv_options = PerInvocation::new();
        inv_options.n_threads = Some(8);
        let executor = LlamaExecutor::new_with_options(Some(exec_options), Some(inv_options))
            .map_err(|_| LLMError::InitializingLLMFailed)?;

        Ok(Self { exec: executor })
    }

    // Submit a prompt to the LLM if it isn't currently busy
    pub async fn submit_prompt(&mut self) -> Result<String, LLMError> {
        println!("Prompt received, submitting to LLM...");
        // Run prompt
        let res = prompt!("Write a hypothetical weather report for {season} in {location}.")
            .run(
                &parameters!("season" => "summer", "location" => "the moon"),
                &self.exec,
            )
            .await
            .map_err(|_| LLMError::SubmittingPromptFailed)?;
        // Acquire result string
        let res_string = res.to_string();
        println!("Local LLM Response: {}", res_string);

        // Return string
        return Ok(res_string);
    }
}
