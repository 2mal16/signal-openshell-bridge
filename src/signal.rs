use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use reqwest;

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
        let event = event?;
        println!("Received SSE event: {}", event.data);
    }

    Ok(())
}
