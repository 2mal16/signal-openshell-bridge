use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub bind_ip: IpAddr,
    pub gateway: SocketAddr,
    pub listen: SocketAddr,
    pub allowed_phones: Vec<PhoneNumber>,
}
