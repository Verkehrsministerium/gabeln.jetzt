use std::error::Error;

#[derive(Debug)]
pub enum GabelnError {
    FailedToFetchUserEvents(String),
    FailedToParseUserEvents,
    FailedToCreateFeed,
    NoTelegramBotToken,
    FailedToCreateTelegramBot,
    FailedToListenForTelegramMessages,
}

impl Error for GabelnError {
    fn description(&self) -> &str {
        match *self {
            GabelnError::FailedToFetchUserEvents(_) => "Failed to fetch the events for the given user!",
            GabelnError::FailedToParseUserEvents => "Failed to parse the user events response body!",
            GabelnError::FailedToCreateFeed => "Failed to create atom feed from user events!",
            GabelnError::NoTelegramBotToken => "Please provide a telegram bot token via environment variable!",
            GabelnError::FailedToCreateTelegramBot => "Could not create Telegram API instance!",
            GabelnError::FailedToListenForTelegramMessages => "Could not listen for Telegram messages!",
        }
    }
}

impl std::fmt::Display for GabelnError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            GabelnError::FailedToFetchUserEvents(ref user) => write!(
                f, "Failed to fetch the events for the user {}!", user
            ),
            _ => write!(f, "{}", self.description()),
        }
    }
}
