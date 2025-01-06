/// Sendet eine Anfrage an den Ollama-Server mit einem Prompt.
use reqwest::Client;
use serde_json::{json, Value};
//use std::fs;
//use std::io;
//use tokio::io::AsyncReadExt;
use crate::shared::Config;
use tokio_stream::StreamExt;

pub async fn send_prompt(
    client: &Client,
    config: &Config,
    prompt: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = json!({
        "model": config.model, // Modell anpassen
        "prompt": prompt
    });

    let response_result = client.post(config.url.clone()).json(&payload).send().await;

    match response_result {
        Ok(response) => {
            if !response.status().is_success() {
                eprintln!("Fehlerhafte Antwort vom Server: HTTP {}", response.status());
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Keine Fehlermeldung verfügbar".to_string());
                eprintln!("Server-Fehlermeldung: {}", error_text);
            } else {
                let mut stream = response.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    if let Ok(text) = String::from_utf8(chunk.to_vec()) {
                        for line in text.lines() {
                            if let Ok(json) = serde_json::from_str::<Value>(line) {
                                if let Some(response) = json.get("response") {
                                    print!("{}", response.as_str().unwrap_or(""));
                                }
                            }
                        }
                    }
                }
                println!(); // Neue Zeile nach der vollständigen Antwort
            }
        }
        Err(err) => {
            eprintln!("Fehler beim Aufruf der URL: {}", err);
        }
    }

    Ok(())
}
