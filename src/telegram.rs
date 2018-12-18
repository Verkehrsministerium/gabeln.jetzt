use std::env;
use std::collections::HashMap;
use futures::{Future, Stream, future::{ok, lazy}, sync::mpsc::Receiver};
use telegram_bot_fork::{
    Api,
    CanReplySendMessage,
    CanSendDocument,
    CanSendMessage,
    CanGetChatAdministrators,
    GetMe,
    GetChatAdministrators,
    Message,
    MessageChat,
    MessageChat::Private,
    MessageChat::Group,
    MessageChat::Supergroup,
    MessageKind,
    ParseMode,
    Update,
    UpdateKind,
    User,
};
use error::GabelnError;
use events::Event;
use giphy::Giphy;
use regex::Regex;

#[derive(Clone)]
pub struct TelegramBot {
    api: Api,
    chats: HashMap<MessageChat, Vec<User>>,
    me: User,
    giphy: Giphy,
    command_re: Regex,
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

        let me = tokio::runtime::current_thread::Runtime::new().unwrap()
            .block_on(api.send(GetMe))
            .map_err(|_| GabelnError::FailedToGetOwnUser)?;

        Ok(Self {
            api: api,
            chats: HashMap::new(),
            me: me,
            giphy: giphy,
            command_re: Regex::new(r"^/(\w+)(?:@(\w+))?$").unwrap(),
        })
    }

    pub fn run<'a>(mut self, recv: Receiver<Event>) -> Result<(), GabelnError> {
        let update_stream = self.api.stream()
            .map(|update| BotUpdate::Update(update))
            .map_err(|_| GabelnError::FailedToListenForTelegramMessages);

        let event_stream = recv
            .map(|event| BotUpdate::Event(event))
            .map_err(|_| GabelnError::FailedToListenForEvents);

        tokio::runtime::current_thread::Runtime::new().unwrap().block_on(lazy(|| {
            info!("Running telegram bot");

            update_stream
                .select(event_stream)
                .for_each(|bot_update| self.on_event(bot_update))
        }))
    }

    fn on_event<'a>(&'a mut self, bot_update: BotUpdate) -> Box<Future<Item = (), Error = GabelnError> + 'a> {
        match bot_update {
            BotUpdate::Update(update) => {
                self.update(update)
            },
            BotUpdate::Event(event) => {
                self.send_text(format!(
                    "**{}** forked __{}__ at [{}]({})!",
                    event.actor.display_login,
                    event.repo.name,
                    event.payload.forkee.clone().unwrap().full_name,
                    event.payload.forkee.clone().unwrap().html_url,
                ))
            },
        }
    }

    fn send_gif<'a>(&'a self, reply_chat: Option<MessageChat>) -> Box<Future<Item = (), Error = GabelnError> + 'a> {
        Box::new(lazy(move || {
            let url = self.giphy.get_gif()?;

            if let Some(chat) = reply_chat {
                self.api.spawn(chat.document_url(url));
            } else {
                for chat in self.chats.keys() {
                    self.api.spawn(chat.document_url(url.clone()));
                }
            }

            Ok(())
        }))
    }

    fn send_text<'a>(&'a self, msg: String) -> Box<Future<Item = (), Error = GabelnError> + 'a> {
        Box::new(lazy(move || {
            for chat in self.chats.keys() {
                self.api.spawn(chat.text(&msg).parse_mode(ParseMode::Markdown));
            }

            self.send_gif(None)
        }))
    }

    fn update<'a>(&'a mut self, update: Update) -> Box<Future<Item = (), Error = GabelnError> + 'a> {
        Box::new(lazy(move || {
            if let UpdateKind::Message(ref message) = update.kind {
                match message.kind {
                    MessageKind::LeftChatMember {ref data, ..} => {
                        if data.id == self.me.id {
                            info!("Stopping bot in chat: {}", &message.chat.id());
                            self.chats.remove(&message.chat);
                        }

                        Box::new(ok(()))
                    },
                    MessageKind::Text {ref data, ..} => {
                        let message = message.clone();
                        let content = data.clone();
                        match self.command(content.as_str(), &message.chat) {
                            Some("start") => {
                                self.cmd_start(message)
                            },
                            Some("stop") => {
                                self.cmd_stop(message)
                            },
                            Some(command) => {
                                self.cmd_unknown(message, command.to_owned())
                            },
                            None => {
                                if self.chats.contains_key(&message.chat) && content.contains("gabeln.jetzt") {
                                    info!("User {} requested gif", message.from.first_name);
                                    return self.send_gif(Some(message.chat));
                                }

                                return Box::new(ok(()));
                            },
                        }
                    },
                    _ => {
                        Box::new(ok(()))
                    },
                }
            } else {
                Box::new(ok(()))
            }
        }))
    }

    fn cmd_start<'a, 'b>(&'a mut self, message: Message) -> Box<Future<Item = (), Error = GabelnError> + 'a> {
        debug!("Trying to start bot!");
        if self.check_admin(&message) {
            debug!("User is authorized!");
            if self.chats.contains_key(&message.chat) {
                self.api.spawn(message.text_reply("Bot already running in this chat!"));
            } else {
                info!("Starting bot in new chat: {}", message.chat.id());
                self.api.spawn(message.text_reply("Starting bot in this chat!"));

                let get_admins: Option<GetChatAdministrators> = match message.chat {
                    Group(ref group) => Some(group.get_administrators()),
                    Supergroup(ref group) => Some(group.get_administrators()),
                    _ => None,
                };

                if let Some(get_admins) = get_admins {
                    return Box::new(
                        self.api
                            .send(get_admins)
                            .map(move |admins| {
                                debug!("Admins: {:?}", admins);

                                self.chats.insert(
                                    message.chat.clone(),
                                    admins
                                        .iter()
                                        .map(|member| member.user.clone())
                                        .collect(),
                                );
                            })
                            .map_err(|_| GabelnError::FailedToStartBot)
                    );
                }
            }
        }

        return Box::new(ok(()));
    }

    fn cmd_stop<'a>(&'a mut self, message: Message) -> Box<Future<Item = (), Error = GabelnError> + 'a> {
        Box::new(lazy(move || {
            debug!("Trying to stop bot!");
            if self.check_admin(&message) {
                if self.chats.contains_key(&message.chat) {
                    info!("Stopping bot in chat: {}", message.chat.id());
                    self.chats.remove(&message.chat);
                    self.api.spawn(message.text_reply("Stopping bot in this chat!"));
                } else {
                    self.api.spawn(message.text_reply("Bot is not running in this chat!"));
                }
            }

            Ok(())
        }))
    }

    fn cmd_unknown<'a>(&'a self, message: Message, command: String) -> Box<Future<Item = (), Error = GabelnError> + 'a> {
        Box::new(lazy(move || {
            warn!("Unknown command {}!", command);
            self.api.spawn(message.text_reply(format!("Invalid command `{}`!", command)));

            Ok(())
        }))
    }

    fn command<'a>(&self, content: &'a str, chat: &MessageChat) -> Option<&'a str> {
        debug!("Parsing command {}", content);
        if let Some(captures) = self.command_re.captures(content) {
            debug!("Found captures {:?}", captures);
            if let Some(receiver) = captures.get(2) {
                debug!("Found receiver {}", receiver.as_str());
                if let Some(ref username) = self.me.username {
                    debug!("Found username {}", username);
                    if receiver.as_str() == username.as_str() {
                        debug!("Matches!");
                        return captures.get(1).map(|c| c.as_str());
                    }
                }
            } else if let Private(_) = chat {
                return captures.get(1).map(|c| c.as_str());
            }
        }

        None
    }

    fn check_admin(&self, message: &Message) -> bool {
        let authorized = match message.chat {
            Private(_) => true,
            _ => {
                debug!("Cached admins: {:?}", self.chats);
                self.chats
                    .get(&message.chat)
                    .map(|admins| {
                        admins
                            .iter()
                            .any(|item| *item == message.from)
                    })
                    .unwrap_or(false)
            },
        };

        if !authorized {
            self.api.spawn(message.text_reply(
                "You are not authorized to do this! Only administrators are allowed to use this command!"
            ));
        }

        authorized
    }
}
