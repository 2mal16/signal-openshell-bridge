use crate::message::SignalEvent;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use reqwest;
use tracing_subscriber::reload::Error;

pub fn process_event(event: SignalEvent) -> Result<(), Error> {
    println!("{event:#?}");
    Ok(())
}

pub async fn listen(events_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::Client::new()
        .get(events_url)
        .header("Accept", "text/event-stream")
        .send()
        .await?
        .error_for_status()?;

    println!("Connected to signal-cli");

    let mut stream = response.bytes_stream().eventsource();

    while let Some(event) = stream.next().await {
        let event: SignalEvent = match serde_json::from_str(&event?.data) {
            Ok(parsed) => parsed,
            Err(error) => {
                eprintln!("Failed to deserialize Signal event: {error}");
                continue;
            }
        };
        process_event(event);
        // let parsed_string = from_str(&event.data);
    }

    Ok(())
}
