mod config;

use config::Config;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&text)?;
    println!("{config:#?}");
    Ok(())
}
