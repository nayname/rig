// src/llm.rs
use serde_json::Value;

/// Ask the LLM and return (request, response)
pub fn ask_gpt(messages: &Value) -> (Value, Value) {
    // …real client code here…
    unimplemented!("Integrate with an LLM provider (e.g. OpenAI) and return (request, response)")
}

/// Build a single-message context for the LLM
pub fn get_context(query: &str, context: &str) -> Value {
    serde_json::json!([{ "role": "user",
        "content": format!("Context: {context}\n\nUser: {query}") }])
}
