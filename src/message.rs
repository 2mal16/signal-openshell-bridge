use crate::config::PhoneNumber;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypingAction {
    Started,
    Stopped,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TypingMessage {
    pub action: TypingAction,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DataMessage {
    timestamp: i64,
    pub message: String,
    expires_in_seconds: i8,
    view_once: bool,
    is_end_session: bool,
    has_profile_key: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Envelope {
    source: PhoneNumber,
    pub source_number: PhoneNumber,
    source_uuid: Uuid,
    pub source_name: String,
    source_device: u16,
    pub timestamp: i64,
    server_received_timestamp: i64,
    server_delivered_timestamp: i64,
    pub typing_message: Option<TypingMessage>,
    pub data_message: Option<DataMessage>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SignalEvent {
    pub envelope: Envelope,
    pub account: PhoneNumber,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthorizedMessage {
    pub id: i64,
    pub sender: String,
    pub message: String,
}
