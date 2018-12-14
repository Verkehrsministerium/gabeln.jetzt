use std::env;
use futures::{Stream, future::lazy, sync::mpsc::Receiver};
use telegram_bot_fork::{
    Api,
    CanReplySendMessage,
    CanSendDocument,
    CanSendMessage,
    GetMe,
    MessageChat,
    MessageKind,
    ParseMode,
    Update,
    UpdateKind,
    User,
};
use error::GabelnError;
use events::Event;
use giphy::Giphy;

pub struct TelegramBot {
    api: Api,
    chats: Vec<MessageChat>,
    me: User,
    giphy: Giphy,
}

enum BotUpdate {
    Update(Update),
    Event(Event),
}

impl TelegramBot {
    pub fn new() -> Result<TelegramBot, GabelnError> {
        let giphy = Giphy::new()?;
        let token = env::var("TELEGRAM_BOT_TOKEN")
            .map_err(|_| GabelnError::NoTelegramBotToken)?;
        let api = Api::new(token)
            .map_err(|_| GabelnError::FailedToCreateTelegramBot)?;

        let me = tokio::runtime::current_thread::Runtime::new().unwrap().block_on(lazy(|| {
            api.send(GetMe)
        })).map_err(|_| GabelnError::FailedToGetOwnUser)?;

        Ok(Self {
            api: api,
            chats: vec![],
            me: me,
            giphy: giphy,
        })
    }

    pub fn run(mut self, recv: Receiver<Event>) -> Result<(), GabelnError> {
        tokio::runtime::current_thread::Runtime::new().unwrap().block_on(lazy(|| {
            let event_stream = recv
                .map(|event| BotUpdate::Event(event))
                .map_err(|_| GabelnError::FailedToListenForEvents);

            self.api.stream()
                .map(|update| BotUpdate::Update(update))
                .map_err(|_| GabelnError::FailedToListenForTelegramMessages)
                .select(event_stream)
                .for_each(|bot_update| {
                    match bot_update {
                        BotUpdate::Update(update) => {
                            self.update(update);
                        },
                        BotUpdate::Event(event) => {
                            self.send_text(format!(
                                "**{}** forked __{}__ at [{}]({})!",
                                event.actor.display_login,
                                event.repo.name,
                                event.payload.forkee.clone().unwrap().full_name,
                                event.payload.forkee.clone().unwrap().html_url,
                            ));
                        },
                    }

                    Ok(())
                })
        }))
    }

    fn send_gif(&self) {
        for chat in self.chats.iter() {
            self.api.spawn(chat.document_url(self.giphy.get_gif().unwrap()));
        }
    }

    fn send_text(&self, msg: String) {
        for chat in self.chats.iter() {
            self.api.spawn(chat.text(&msg).parse_mode(ParseMode::Markdown));
            self.send_gif();
        }
    }

    fn update(&mut self, update: Update) {
        if let UpdateKind::Message(message) = update.kind {
            match message.kind {
                MessageKind::LeftChatMember {ref data, ..} => {
                    if data.id == self.me.id {
                        info!("Stopping bot in chat: {}", message.chat.id());
                        self.chats.remove_item(&message.chat);
                    }
                },
                MessageKind::Text {ref data, ..} => {
                    match data.as_str() {
                        "/start" => {
                            if self.chats.contains(&message.chat) {
                                self.api.spawn(message.text_reply("Bot already running in this chat!"));
                            } else {
                                info!("Starting bot in new chat: {}", message.chat.id());
                                self.api.spawn(message.text_reply("Starting bot in this chat!"));
                                self.chats.push(message.chat);
                            }
                        },
                        "/stop" => {
                            if self.chats.contains(&message.chat) {
                                info!("Stopping bot in chat: {}", message.chat.id());
                                self.chats.remove_item(&message.chat);
                                self.api.spawn(message.text_reply("Stopping bot in this chat!"));
                            } else {
                                self.api.spawn(message.text_reply("Bot is not running in this chat!"));
                            }
                        },
                        _ => {
                            if self.chats.contains(&message.chat) && data.contains("gabeln.jetzt") {
                                info!("User {} requested gif", message.from.first_name);
                                self.send_gif();
                            }
                        },
                    }
                },
                _ => {},
            }
        }
    }
}
