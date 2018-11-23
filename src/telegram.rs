use std::env;
use futures::{Future, Stream, future::lazy};
use telegram_bot_fork::{Api, UpdateKind, MessageKind, CanReplySendMessage};
use error::GabelnError;

pub struct TelegramBot {
    api: Api,
}

impl TelegramBot {
    pub fn new() -> Result<TelegramBot, GabelnError> {
        let token = env::var("TELEGRAM_BOT_TOKEN")
            .map_err(|_| GabelnError::NoTelegramBotToken)?;
        let api = Api::new(token)
            .map_err(|_| GabelnError::FailedToCreateTelegramBot)?;

        Ok(Self {
            api: api,
        })
    }

    pub fn run(self) -> Result<(), GabelnError> {
        // TODO: run in background thread
        tokio::runtime::current_thread::Runtime::new().unwrap().block_on(lazy(|| {
            self.api.stream().for_each(|update| {
                // if the received update contains a new message...
                if let UpdateKind::Message(message) = update.kind {
                    if let MessageKind::Text { ref data, .. } = message.kind {
                        if data.contains("gabeln.jetzt") {
                            self.api.spawn(message.text_reply("alles gabelt und nix löffelt"));
                        }
                    }
                }

                Ok(())
            }).map_err(|_| GabelnError::FailedToListenForTelegramMessages)
        }))
    }
}
