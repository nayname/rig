use std::env;

use rig::pipeline::{self, Op};
use rig::providers::openai;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use rig::providers::openai::client::Client;

#[tokio::main]
pub async fn ask_gpt(messages: &Value) -> &str {
    // Create OpenAI client
    let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let openai_client = Client::new(&openai_api_key);

    let rng_agent = openai_client.agent("gpt-4")
        .preamble("
            You are a random number generator designed to only either output a single whole integer that is 0 or 1. Only return the number.
        ")
        .build();

    let adder_agent = openai_client.agent("gpt-4")
        .preamble("
            You are a mathematician who adds 1000 to every number passed into the context, except if the number is 0 - in which case don't add anything. Only return the number.
        ")
        .build();

    let chain = pipeline::new()
        // Generate a whole number that is either 0 and 1
        .prompt(rng_agent)
        .map(|x| x.unwrap())
        .prompt(adder_agent);

    // Prompt the agent and print the response
    let response = chain
        .call("Please generate a single whole integer that is 0 or 1".to_string())
        .await;

    println!("Pipeline result: {response:?}");

    return "response"
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
/// An enum representing the sentiment of a document
enum Sentiment {
    Positive,
    Negative,
    Neutral,
}

#[derive(Debug, Deserialize, JsonSchema, Serialize)]
struct DocumentSentiment {
    /// The sentiment of the document
    sentiment: Sentiment,
}

#[tokio::main]
pub async fn classify(messages: &Value) -> String {
    // Create OpenAI client
    let openai_client = openai::Client::from_env();

    // Create extractor
    let data_extractor = openai_client
        .extractor::<DocumentSentiment>("gpt-4")
        .build();

    let sentiment = data_extractor
        .extract(messages.to_string())
        .await
        .expect("Failed to extract sentiment");

    println!("GPT-4: {:?}", sentiment);

    match sentiment {
        DocumentSentiment::Positive(text) => {
            // обработка позитивного настроения
        },
        DocumentSentiment::Negative(text) => {
            // обработка негативного настроения
        }
    }

    return "Null"
}

/// Build a single-message context for the LLM
pub fn get_context(query: &str, context: &str) -> Value {
    serde_json::json!([{ "role": "user",
        "content": format!("Context: {context}\n\nUser: {query}") }])
}
