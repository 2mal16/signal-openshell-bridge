use crate::config::Config;
use crate::dispatcher::dispatch;
use crate::message::SignalEvent;
use crate::processor::process_event;

use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use reqwest;

pub async fn listen(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = reqwest::Client::new()
        .get(&config.events_url())
        .header("Accept", "text/event-stream")
        .send()
        .await?
        .error_for_status()?;

    println!("Connected to signal-cli");

    let mut stream = response.bytes_stream().eventsource();

    while let Some(event) = stream.next().await {
        let event: SignalEvent = match serde_json::from_str(&event?.data) {
            Ok(event) => event,
            Err(error) => {
                eprintln!("Failed to deserialize Signal event: {error}");
                continue;
            }
        };
        match process_event(event, config) {
            Ok(message) => match dispatch(&client, config.gateway, &message).await {
                Ok(response) => {
                    println!(
                        "Message {} dispatched  {} successfully",
                        message.id, response
                    );
                }
                Err(error) => {
                    eprintln!("Dispatch failed: {}", error);
                }
            },
            Err(error) => {
                eprintln!("Processing event failed: {}", error);
            }
        }
    }
    Ok(())
}
