mod config;
mod message;
mod signal;

use config::Config;
use signal::listen;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&text)?;
    println!("{config:#?}");
    listen(&config.events_url()).await?;
    Ok(())
}
