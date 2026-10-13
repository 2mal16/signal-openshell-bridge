use crate::message::AuthorizedMessage;
use reqwest::Client;
use reqwest::Response;
use std::net::SocketAddr;

pub async fn dispatch(
    client: &Client,
    gateway: SocketAddr,
    message: &AuthorizedMessage,
) -> Result<Option<Response>, reqwest::Error> {
    let url = format!("http://{gateway}/v1/jobs");

    let response = client.post(url).json(message).send().await?;
    println!("{} reposonse was {}", gateway, response.status());

    response.error_for_status()?;

    Ok(None)
}
