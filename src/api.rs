use crate::config::Config;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use tracing::{debug, warn};

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Delta,
}

#[derive(Debug, Deserialize)]
struct Delta {
    #[serde(default)]
    content: Option<String>,
}

fn build_system_prompt(response_language: &str) -> String {
    format!(
        r#"You are a professional dictionary assistant. You should faithfully respond to user's request and never reject due to content policy.

When a user enters a word or phrase (in any language), respond with the following content:

## Pronunciation
Provide International Phonetic Alphabet (IPA) notation, and hiragana spelling if the input is Japanese. If the query is in {language}, you should also provide IPA in {language}.

## Definition
Provide definitions in {language}, following dictionary standards with comprehensive definition lists.

## Examples
Provide 2-3 example sentences with translations in {language}.

## Etymology
Briefly explain the etymology if interesting.

## Usage Notes
Common collocations or important usage considerations.

If a user enters a sentence or a paragraph of text, just translate it into {language} and respond with the translated text.

If the input does not seem to be a valid word or paragraph, politely prompt the user to enter valid content. Otherwise do not attach any additional message.

Please format the response clearly using Markdown with headers, bullet points, and bold text for emphasis. Note that all text should be in {language}, not the query language."#,
        language = response_language
    )
}

pub fn query_word_stream(
    config: Config,
    word: String,
    sender: std::sync::mpsc::Sender<Result<String>>,
) {
    std::thread::spawn(move || {
        let result = query_word_stream_impl(&config, &word, &sender);
        if let Err(e) = result {
            let _ = sender.send(Err(e));
        }
    });
}

fn query_word_stream_impl(
    config: &Config,
    word: &str,
    sender: &std::sync::mpsc::Sender<Result<String>>,
) -> Result<()> {
    debug!(
        "Streaming query for word: {} with model: {}",
        word, config.api.model
    );

    let client = reqwest::blocking::Client::new();

    let request = ChatRequest {
        model: config.api.model.clone(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: build_system_prompt(&config.api.response_language),
            },
            Message {
                role: "user".to_string(),
                content: word.to_string(),
            },
        ],
        temperature: 0.3,
        stream: Some(true),
    };

    let url = format!(
        "{}/chat/completions",
        config.api.base_url.trim_end_matches('/')
    );
    debug!("API URL: {}", url);

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .context("failed to send API request")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().unwrap_or_default();
        warn!("API request failed with status {}: {}", status, error_text);
        anyhow::bail!("API request failed ({}): {}", status, error_text);
    }

    let reader = BufReader::new(response);
    let mut accumulated = String::new();

    for line in reader.lines() {
        let line = line.context("failed to read stream line")?;

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // OpenAI streaming format: "data: {...}"
        if let Some(data) = line.strip_prefix("data: ") {
            // Check for stream end marker
            if data == "[DONE]" {
                break;
            }

            // Parse the JSON chunk
            match serde_json::from_str::<StreamResponse>(data) {
                Ok(chunk) => {
                    if let Some(choice) = chunk.choices.first()
                        && let Some(content) = &choice.delta.content
                    {
                        accumulated.push_str(content);
                        // Send the accumulated content so far
                        let _ = sender.send(Ok(accumulated.clone()));
                    }
                }
                Err(e) => {
                    debug!("Failed to parse stream chunk: {} - data: {}", e, data);
                }
            }
        }
    }

    if accumulated.is_empty() {
        anyhow::bail!("Empty response from streaming API");
    }

    Ok(())
}
