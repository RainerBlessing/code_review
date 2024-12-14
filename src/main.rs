use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::io;
use tokio::io::AsyncReadExt;
use tokio_stream::StreamExt;

#[derive(Deserialize)]
struct Config {
    model: String,
    url: String,
}

fn load_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config("config.json")?;
    println!("Verwende Modell: {}, URL: {}", config.model, config.url);

    // HTTP-Client erstellen
    let client = Client::new();

    // 1. Initialer Prompt
    let introduction_prompt = "Es folgen einige Quellcode-Dateien. Bitte keine Antwort, bis eine Anweisung gegeben wurde.";
    send_prompt(&client, &config, introduction_prompt).await?;

    // 2. Liste von Dateien sequentiell senden
    let file_paths = vec!["src/main.rs", "src/lib.rs"]; // Passe die Pfade an
    for file_path in file_paths {
        match read_file_to_string(file_path).await {
            Ok(file_content) => {
                let file_prompt = format!(
                    "Hier ist der Inhalt der Datei `{}`:\n\n```rust\n{}\n```",
                    file_path, file_content
                );
                send_prompt(&client, &config, &file_prompt).await?;
            }
            Err(err) => {
                eprintln!("Fehler beim Lesen der Datei {}: {}", file_path, err);
            }
        }
    }

    // 3. Abschließender Prompt für das Review
    let review_prompt = "Bitte führe einen Code-Review für die zuvor übergebenen Quellcode-Dateien durch. Wenn Du einen Vorschlag für Code-Änderungen machst, markiere die geänderten oder ergänzten Stellen. Bitte für die Ausgabe das Markdown-Format verwenden.";
    send_prompt(&client, &config, &review_prompt).await?;

    Ok(())
}

/// Sendet eine Anfrage an den Ollama-Server mit einem Prompt.
async fn send_prompt(
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

/// Liest den Inhalt einer Datei und gibt ihn als String zurück.
async fn read_file_to_string(file_path: &str) -> Result<String, io::Error> {
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut content = String::new();
    file.read_to_string(&mut content).await?;
    Ok(content)
}

