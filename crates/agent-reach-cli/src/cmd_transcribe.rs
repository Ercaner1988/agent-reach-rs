//! Transcribe a local audio/video file through Groq or OpenAI Whisper APIs.

use agent_reach_core::Config;
use anyhow::{bail, Context, Result};
use reqwest::multipart;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct TranscriptionResponse {
    text: String,
}

pub async fn transcribe(source: String, provider: String, output: Option<String>) -> Result<()> {
    let path = Path::new(&source);
    if !path.is_file() {
        bail!("transcription source is not a local file: {}", source);
    }

    let mut config = Config::load().unwrap_or_default();
    let selected = match provider.to_lowercase().as_str() {
        "auto" => {
            if config.groq_api_key.is_some() {
                "groq"
            } else if config.openai_api_key.is_some() {
                "openai"
            } else {
                bail!("no Groq or OpenAI API key configured; use `agent-reach configure groq_api_key <key>`")
            }
        }
        "groq" | "openai" => provider.as_str(),
        other => bail!(
            "unsupported transcription provider '{}'; use auto, groq, or openai",
            other
        ),
    };

    let (endpoint, api_key) = match selected {
        "groq" => (
            "https://api.groq.com/openai/v1/audio/transcriptions",
            config.groq_api_key.take().unwrap_or_default(),
        ),
        "openai" => (
            "https://api.openai.com/v1/audio/transcriptions",
            config.openai_api_key.take().unwrap_or_default(),
        ),
        _ => unreachable!(),
    };

    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("failed to read transcription source: {}", source))?;
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("audio.bin")
        .to_string();

    let form = multipart::Form::new()
        .part("file", multipart::Part::bytes(bytes).file_name(filename))
        .text("model", "whisper-large-v3-turbo")
        .text("response_format", "json");

    let client = reqwest::Client::new();
    let response = client
        .post(endpoint)
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await
        .context("transcription request failed")?;
    let status = response.status();
    let body = response
        .text()
        .await
        .context("failed to read transcription response")?;

    if !status.is_success() {
        bail!("transcription API returned HTTP {}: {}", status, body);
    }

    let transcript: TranscriptionResponse =
        serde_json::from_str(&body).context("transcription API returned invalid JSON")?;
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &transcript.text)
            .await
            .with_context(|| format!("failed to write transcript: {}", output_path))?;
        println!("transcript written to {}", output_path);
    } else {
        println!("{}", transcript.text);
    }

    Ok(())
}
