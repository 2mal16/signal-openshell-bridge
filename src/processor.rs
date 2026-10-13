use crate::config::Config;
use crate::error::Error;
use crate::message::AuthorizedMessage;
use crate::message::SignalEvent;

pub fn authorize_event(event: SignalEvent, config: &Config) -> Result<AuthorizedMessage, Error> {
    if !config.allowed_accounts.contains(&event.account) {
        return Err(Error::NumberNotAllowed(event.account));
    }
    if !config
        .allowed_phones
        .contains(&event.envelope.source_number)
    {
        return Err(Error::NumberNotAllowed(event.envelope.source_number));
    }
    if !config
        .allowed_source_names
        .contains(&event.envelope.source_name)
    {
        return Err(Error::AccountNotAllowed(event.envelope.source_name));
    }
    let Some(data_message) = event.envelope.data_message else {
        return Err(Error::NoMessage());
    };
    if data_message.message.trim().is_empty() {
        return Err(Error::EmptyMessage());
    }
    Ok(AuthorizedMessage {
        id: event.envelope.timestamp,
        sender: event.envelope.source_name,
        message: data_message.message,
    })
}

pub fn process_event(event: SignalEvent, config: &Config) -> Option<AuthorizedMessage> {
    match authorize_event(event, config) {
        Ok(message) => {
            println!(
                "Authorized message received {} from {}",
                message.message, message.sender
            );
            None
        }
        Err(error) => {
            println!("Message was not authorized with error {}", error);
            None
        }
    }
}
