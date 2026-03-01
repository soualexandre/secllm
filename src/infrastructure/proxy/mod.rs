//! Proxy adapter – forward requests to LLM providers (OpenAI/Anthropic).

mod reqwest_dispatcher;

pub use reqwest_dispatcher::ReqwestDispatcher;
