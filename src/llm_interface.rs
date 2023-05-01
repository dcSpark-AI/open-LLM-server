use std::error::Error;
use llm_chain::{parameters, prompt, traits::Executor};
use llm_chain_openai::chatgpt::Executor as ChatGPTExecutor;
use llm_chain_llama::Executor as LlamaGPTExecutor;

pub struct LLMInterface<T: Executor> {
    exec: T,
    is_busy: bool,
}

impl LLMInterface<LlamaGPTExecutor> {
    pub fn with_llama_executor() -> Result<Self, Box<dyn Error>> {
        let executor = LlamaGPTExecutor::new()?;
        Ok(Self { exec: executor, is_busy: false })
    }
    
     pub async fn submit_prompt(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        if (!self.is_busy) {
            // Set self to busy
            self.is_busy = true;

            // Run prompt
            let res = prompt!("Write a hypothetical weather report for {season} in {location}.")
                .run(
                    &parameters!("season" => "summer", "location" => "the moon"),
                    &self.exec,
                )
                .await?;
            let res_string = res.to_string();
            println!("Llama: {}", res_string);
            return Ok(res_string);
        }
        
        // Change this to error
       Ok(String::new())
    }
}

impl LLMInterface<ChatGPTExecutor> {
    pub fn with_chatgpt_executor() -> Result<Self, Box<dyn Error>> {
        let executor = ChatGPTExecutor::new()?;
        Ok(Self { exec: executor, is_busy: false })
    }
}

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

