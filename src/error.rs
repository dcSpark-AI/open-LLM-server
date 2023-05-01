use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum LLMError {
    IsBusy,
    InitializingLLMFailed,
    SubmittingPromptFailed,
    Custom(String),
}

impl fmt::Display for LLMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LLMError::IsBusy => write!(f, "LLM is currently busy."),
            LLMError::InitializingLLMFailed => write!(f, "Initializing the LLLM has failed."),
            LLMError::SubmittingPromptFailed => {
                write!(f, "Submitting prompt to the LLM has failed.")
            }
            LLMError::Custom(s) => {
                write!(f, "{s}")
            }
        }
    }
}

impl From<hyper::http::Error> for LLMError {
    fn from(error: hyper::http::Error) -> Self {
        LLMError::Custom(format!("Hyper HTTP error: {}", error))
    }
}

impl From<serde_json::Error> for LLMError {
    fn from(error: serde_json::Error) -> Self {
        LLMError::Custom(format!("JSON error: {}", error))
    }
}

impl Error for LLMError {}
