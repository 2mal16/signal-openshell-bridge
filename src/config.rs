use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(try_from = "String", into = "String")]
pub struct PhoneNumber(String);

impl TryFrom<String> for PhoneNumber {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let digits = s
            .strip_prefix('+')
            .ok_or("phone number must start with '+'")?;
        if (7..=15).contains(&digits.len()) && digits.chars().all(|c| c.is_ascii_digit()) {
            Ok(PhoneNumber(s))
        } else {
            Err(format!("invalid E.164 phone number: {s}"))
        }
    }
}

impl From<PhoneNumber> for String {
    fn from(p: PhoneNumber) -> String {
        p.0
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub signal_cli: SocketAddr,
    pub gateway: SocketAddr,
    pub allowed_phones: Vec<PhoneNumber>,
    pub allowed_accounts: Vec<PhoneNumber>,
    pub allowed_source_names: Vec<String>,
}

impl Config {
    pub fn events_url(&self) -> String {
        format!("http://{}/api/v1/events", self.signal_cli)
    }
}
