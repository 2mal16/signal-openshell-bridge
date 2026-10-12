use crate::config::PhoneNumber;
use serde::Deserialize;
use serde_json::Number;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypingAction {
    Started,
    Stopped,
}

#[derive(Debug, Deserialize)]
pub struct TypingMessage {
    action: TypingAction,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataMessage {
    timestamp: i64,
    message: String,
    expires_in_seconds: i8,
    view_once: bool,
    is_end_session: bool,
    has_profile_key: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope {
    source: PhoneNumber,
    source_number: PhoneNumber,
    source_uuid: Uuid,
    source_name: String,
    source_device: Number,
    timestamp: i64,
    server_received_timestamp: i64,
    server_delivered_timestamp: i64,
    typing_message: Option<TypingMessage>,
    data_message: Option<DataMessage>,
}

#[derive(Debug, Deserialize)]
pub struct SignalEvent {
    envelope: Envelope,
    account: PhoneNumber,
}

#[derive(Debug, Deserialize)]
pub struct ReceivedEvent {
    pub sse_id: String,
    pub message: SignalEvent,
}

pub struct Message {
    id: u16,
    message: String,
}

pub struct AllowedMessage {
    id: u16,
    message: String,
}
