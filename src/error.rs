use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum LLMError {
    IsBusy,
    InitializingLLMFailed,
    SubmittingPromptFailed,
}

impl fmt::Display for LLMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LLMError::IsBusy => write!(f, "LLM is currently busy."),
            LLMError::InitializingLLMFailed => write!(f, "Initializing the LLLM has failed."),
            LLMError::SubmittingPromptFailed => {
                write!(f, "Submitting prompt to the LLM has failed.")
            }
        }
    }
}

impl Error for LLMError {}
