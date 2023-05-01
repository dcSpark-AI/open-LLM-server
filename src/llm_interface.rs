use crate::error::LLMError;
use llm_chain::{parameters, prompt, traits::Executor};
use llm_chain_llama::Executor as LlamaExecutor;
use llm_chain_openai::chatgpt::Executor as ChatGPTExecutor;
use std::error::Error;

pub struct LLMInterface<T: Executor> {
    exec: T,
    is_busy: bool,
}
impl LLMInterface<LlamaExecutor> {
    pub fn with_llama_executor() -> Result<Self, LLMError> {
        let executor = LlamaExecutor::new().map_err(|_| LLMError::InitializingLLMFailed)?;

        Ok(Self {
            exec: executor,
            is_busy: false,
        })
    }

    // Submit a prompt to the LLM if it isn't currently busy
    pub async fn submit_prompt(&mut self) -> Result<String, LLMError> {
        // If the LLM isn't currently busy
        if !self.is_busy {
            // Set self to busy
            self.is_busy = true;
            // Run prompt
            let res = prompt!("Write a hypothetical weather report for {season} in {location}.")
                .run(
                    &parameters!("season" => "summer", "location" => "the moon"),
                    &self.exec,
                )
                .await
                .map_err(|_| LLMError::InitializingLLMFailed)?;
            // Acquire result string
            let res_string = res.to_string();
            println!("Llama: {}", res_string);

            // Unset busy and return string
            self.is_busy = false;
            return Ok(res_string);
        }

        // Return IsBusy error
        Err(LLMError::IsBusy)
    }
}

// impl LLMInterface<ChatGPTExecutor> {
//     pub fn with_chatgpt_executor() -> Result<Self, Box<dyn Error>> {
//         let executor = ChatGPTExecutor::new()?;
//         Ok(Self { exec: executor, is_busy: false })
//     }
// }

// Goal will be to have this be generic, but upstream library seems to not implement everything we need for genericity in types
// impl<T: Executor> LLMInterface<T> {
//     impl<T: Executor> LLMInterface<T> where T::Output: ToString {
//         pub async fn submit_prompt(&self) -> Result<String, Box<dyn std::error::Error>> {
//             let res = prompt!("Write a hypothetical weather report for {season} in {location}.")
//                 .run(
//                     &parameters!("season" => "summer", "location" => "the moon"),
//                     &self.exec,
//                 )
//                 .await?;
//             let res_string = res.to_string();
//             println!("Llama: {}", res_string);
//             Ok(res_string)
//         }
//     }

// }
