use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use rand::Rng;
use serde_json::{json, Value};

mod llm;

use crate::llm::{ask_gpt, get_context};

//---------------------------------------------------------------------
// CONSTANTS (lifted directly from the Python source)
//---------------------------------------------------------------------

/// List of supported ADO operation classes.
const OPERATIONS: [&str; 7] = [
    "nft_marketplace",
    "crowdfund",
    "cw20_exchange",
    "auction_using_cw20_tokens",
    "extended_marketplace",
    "commission_based_sales",
    "vesting_and_staking",
];

/// Prompt template for query‐classification calls to the LLM.
const CLASSIFY_QUERY: &str = "Lets pretend that we have an LLM app that generates Andromeda Protocol app contracts \
 using user promtps in natural language. You will be given a user's promt. Based on the context, classify the query \
 to one of the following classes. Classes: ***OPERATIONS***. User's query: ***QUERY***";

/// Prompt template for schema‑generation calls to the LLM.
const GENERATE_FLEX: &str = "You will be given a description of the modules and the schema of the modules. Based on this context and the \
 user's query, generate the schema that fulfills the users intent. User's query: ***QUERY***";

//---------------------------------------------------------------------
// HELPER UTILITIES
//---------------------------------------------------------------------

/// Extract parameters from a free‑form query.  (Not yet implemented.)
fn parse_query(_query: &str) -> Value {
    Value::Null
}

/// Convert query parameters back to a prompt context string.  (Not yet implemented.)
fn context_from_params(_params: &Value) -> String {
    String::new()
}

/// Persist the raw LLM answer for later inspection, mirroring the Python version.
fn validate(class_: &str, answer: &Value) -> std::io::Result<()> {
    let mut rng = rand::rng();
    let hash: u128 = rng.random();

    fs::create_dir_all("generated")?;
    let filename = format!("generated/{}_{}", hash, class_);
    let mut file = File::create(filename)?;

    if let Some(content) = answer["choices"][0]["message"]["content"].as_str() {
        file.write_all(content.as_bytes())?;
    }
    Ok(())
}

//---------------------------------------------------------------------
// CORE GENERATION LOGIC (port of `generate()` in Python)
//---------------------------------------------------------------------

fn generate(query: &str, map: &mut Vec<Value>) -> anyhow::Result<()> {
    // 1. Read global context blob from disk.
    let context = fs::read_to_string("context.json")?;

    // 2. CLASSIFICATION STAGE ------------------------------------------------
    let messages = get_context(
        &CLASSIFY_QUERY
            .replace("***OPERATIONS***", &serde_json::to_string(&OPERATIONS)?)
            .replace("***QUERY***", query),
        &context,
    );

    let (_request, answer_class) = ask_gpt(&messages);
    let class_ = answer_class["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .trim_matches('"')
        .to_string();

    println!("Class: {}", class_);

    // 3. CONFIGURATION ASSEMBLY ---------------------------------------------
    let classes_config: Value = serde_json::from_str(&fs::read_to_string("config/config_all.json")?)?;
    let mut prog_context = json!({ "ados_components": [] });

    if let Some(class_def) = classes_config.get(&class_) {
        // Collect component schemas.
        if let Some(component_names) = class_def["classes"].as_array() {
            for name_val in component_names {
                if let Some(name) = name_val.as_str() {
                    let path = format!("config/objects/{}.json", name);
                    let comp_schema = fs::read_to_string(path)?;
                    prog_context["ados_components"].as_array_mut().unwrap().push(Value::String(comp_schema));
                }
            }
        }
        // Attach human‑readable description.
        if let Some(descr) = class_def["descr"].as_str() {
            prog_context["application_description"] = Value::String(descr.to_string());
        }
    }

    // 4. GENERATION STAGE ----------------------------------------------------
    let gen_messages = get_context(
        &GENERATE_FLEX.replace("***QUERY***", query),
        &prog_context.to_string(),
    );
    let (_req2, answer_gen) = ask_gpt(&gen_messages);

    validate(&class_, &answer_gen)?;

    // 5. UPDATE UI MAP -------------------------------------------------------
    let mut rng = rand::rng();
    let hash: u128 = rng.random();
    map.push(json!({
        "name": format!("{}_{}", hash, class_),
        "query": query,
        "label": class_,
    }));

    fs::write("generated_map.json", serde_json::to_string_pretty(&map)?)?;
    Ok(())
}

//---------------------------------------------------------------------
// ENTRY‑POINT
//---------------------------------------------------------------------

fn main() -> anyhow::Result<()> {
    println!("Start");
    // Load the synthetic query set.
    let queries: Value = serde_json::from_str(&fs::read_to_string("queries.json")?)?;

    // Load or initialise the generated‑script index.
    let mut map: Vec<Value> = if Path::new("generated_map.json").exists() {
        serde_json::from_str(&fs::read_to_string("generated_map.json")?)?
    } else {
        Vec::new()
    };

    // Dispatch sample generations (mirrors Python order).
    generate(queries["nft_marketplace"][0].as_str().unwrap(), &mut map)?;
    generate(queries["crowdfund"][0].as_str().unwrap(), &mut map)?;
    generate(queries["cw20_exchange"][0].as_str().unwrap(), &mut map)?;
    generate(queries["auction_using_cw20_tokens"][0].as_str().unwrap(), &mut map)?;
    generate(queries["extended_marketplace"][0].as_str().unwrap(), &mut map)?;
    generate(queries["commission_based_sales"][0].as_str().unwrap(), &mut map)?;
    generate(queries["vesting_and_staking"][0].as_str().unwrap(), &mut map)?;
    generate(queries["extended_marketplace"][1].as_str().unwrap(), &mut map)?;
    generate(queries["cw20_exchange"][1].as_str().unwrap(), &mut map)?;
    generate(queries["vesting_and_staking"][1].as_str().unwrap(), &mut map)?;

    Ok(())
}

//---------------------------------------------------------------------
// DEPENDENCY HINTS (add to Cargo.toml)
//---------------------------------------------------------------------
// [dependencies]
// anyhow = "1"
// rand = "0.8"
// serde = { version = "1", features = ["derive"] }
// serde_json = "1"
