use crate::message::AuthorizedMessage;
use reqwest::Client;
use std::error::Error;
use std::net::SocketAddr;

pub async fn dispatch(
    client: &Client,
    gateway: SocketAddr,
    message: &AuthorizedMessage,
) -> Result<String, reqwest::Error> {
    let url = format!("http://{gateway}/v1/jobs");

    println!("Sending message {} to {}", message.id, url);

    let response = match client.post(&url).json(message).send().await {
        Ok(response) => response,

        Err(error) => {
            eprintln!("Request to {url} failed");
            eprintln!("Full error: {error:#?}");
            eprintln!("Connection error: {}", error.is_connect());
            eprintln!("Timeout: {}", error.is_timeout());
            eprintln!("Request error: {}", error.is_request());
            eprintln!("Builder error: {}", error.is_builder());

            // Walk through the underlying error chain
            let mut source = error.source();

            while let Some(cause) = source {
                eprintln!("Caused by: {cause}");
                source = cause.source();
            }

            return Err(error);
        }
    };

    // Inspect the response
    let status = response.status();

    println!("Status: {status}");
    println!("Headers: {:#?}", response.headers());

    // Read the response body
    let body = response.text().await?;

    println!("Body: {body:?}");

    // Treat non-2xx responses as errors
    if !status.is_success() {
        eprintln!("Server returned unsuccessful HTTP status: {status}");
    }

    Ok(body)
}
