use std::env;
use std::vec::Vec;
use std::sync::{Arc, Mutex};
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
struct InnerTelegramBot {
    api: Api,
    chats: HashMap<MessageChat, Vec<User>>,
    active_chats: Vec<MessageChat>,
    me: User,
    giphy: Giphy,
    command_re: Regex,
}

#[derive(Clone)]
pub struct TelegramBot {
    inner: Arc<Mutex<InnerTelegramBot>>,
}

enum BotUpdate {
    Update(Update),
    Event(Event),
}

type BotFuture<'a> = Box<Future<Item = (), Error = GabelnError> + 'a>;

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

        let inner = InnerTelegramBot {
            api: api,
            chats: HashMap::new(),
            active_chats: Vec::new(),
            me: me,
            giphy: giphy,
            command_re: Regex::new(r"^/(\w+)(?:@(\w+))?$").unwrap(),
        };

        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    pub fn run<'a>(mut self, recv: Receiver<Event>) -> Result<(), GabelnError> {
        let update_stream = self.inner.lock().unwrap().api.stream()
            .map(|update| BotUpdate::Update(update))
            .map_err(|_| GabelnError::FailedToListenForTelegramMessages);

        let event_stream = recv
            .map(|event| BotUpdate::Event(event))
            .map_err(|_| GabelnError::FailedToListenForEvents);

        tokio::runtime::current_thread::Runtime::new().unwrap().block_on(lazy(|| {
            info!("Running telegram bot");

            update_stream
                .select(event_stream)
                .for_each(move |bot_update| self.on_event(bot_update))
        }))
    }

    fn on_event<'a>(&mut self, bot_update: BotUpdate) -> BotFuture<'a> {
        match bot_update {
            BotUpdate::Update(update) => {
                let update_future = self.update(update.clone());
                Box::new(
                    self.register_chat(update)
                        .and_then(move |()| update_future)
                )
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

    fn register_chat<'a>(&mut self, update: Update) -> BotFuture<'a> {
        let inner = self.inner.lock().unwrap();

        let chat = match update.kind {
            UpdateKind::Message(msg) => Some(msg.chat),
            UpdateKind::EditedMessage(msg) => Some(msg.chat),
            _ => None,
        };

        if let Some(chat) = chat {
            if !inner.chats.contains_key(&chat) {
                let get_admins: Option<GetChatAdministrators> = match chat {
                    Group(ref group) => Some(group.get_administrators()),
                    Supergroup(ref group) => Some(group.get_administrators()),
                    _ => None,
                };

                if let Some(get_admins) = get_admins {
                    let inner_arc = self.inner.clone();

                    debug!("Trying to get admins...");

                    return Box::new(
                        inner.api
                            .send(get_admins)
                            .map(move |admins| {
                                let mut inner = inner_arc.lock().unwrap();

                                inner.chats.insert(
                                    chat,
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

        Box::new(ok(()))
    }

    fn update<'a>(&mut self, update: Update) -> BotFuture<'a> {
        if let UpdateKind::Message(ref message) = update.kind {
            match message.kind {
                MessageKind::LeftChatMember {ref data, ..} => {
                    let inner_arc = self.inner.clone();
                    let mut inner = inner_arc.lock().unwrap();

                    if data.id == inner.me.id {
                        info!("Stopping bot in chat: {}", &message.chat.id());
                        inner.active_chats.remove_item(&message.chat);
                        inner.chats.remove(&message.chat);
                    }

                    Box::new(ok(())) as BotFuture<'a>
                },
                MessageKind::Text {ref data, ..} => {
                    let message = message.clone();
                    let content = data.clone();
                    let parsed = self.inner
                        .lock()
                        .unwrap()
                        .command(content.as_str(), &message.chat);

                    match parsed {
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
                            self.cmd_text(message, content.clone())
                        },
                    }
                },
                _ => {
                    Box::new(ok(())) as BotFuture<'a>
                },
            }
        } else {
            Box::new(ok(())) as BotFuture<'a>
        }
    }

    fn send_text<'a>(&self, msg: String) -> BotFuture<'a> {
        let inner_arc = self.inner.clone();

        Box::new(lazy(move || {
            let inner = inner_arc.lock().unwrap();

            for chat in inner.active_chats.iter() {
                inner.api.spawn(chat.text(&msg).parse_mode(ParseMode::Markdown));
            }

            inner.send_gif(None)
        }))
    }

    fn cmd_text<'a>(&self, message: Message, content: String) -> BotFuture<'a> {
        let inner_arc = self.inner.clone();
        Box::new(lazy(move || {
            let inner = inner_arc.lock().unwrap();

            if inner.active_chats.contains(&message.chat) && content.contains("gabeln.jetzt") {
                info!("User {} requested gif", message.from.first_name);
                return inner.send_gif(Some(message.chat));
            }

            Ok(())
        }))
    }

    fn cmd_start<'a>(&self, message: Message) -> BotFuture<'a> {
        let inner_arc = self.inner.clone();

        Box::new(lazy(move || {
            debug!("Trying to start bot!");
            let mut inner = inner_arc.lock().unwrap();

            if inner.check_admin(&message) {
                debug!("User is authorized!");

                if inner.active_chats.contains(&message.chat) {
                    inner.api.spawn(message.text_reply("Bot already running in this chat!"));
                } else {
                    info!("Starting bot in new chat: {}", message.chat.id());
                    inner.api.spawn(message.text_reply("Starting bot in this chat!"));
                    inner.active_chats.push(message.chat);
                }
            }

            Ok(())
        }))
    }

    fn cmd_stop<'a>(&self, message: Message) -> BotFuture<'a> {
        let inner_arc = self.inner.clone();

        Box::new(lazy(move || {
            let mut inner = inner_arc.lock().unwrap();

            debug!("Trying to stop bot!");
            if inner.check_admin(&message) {
                if inner.active_chats.contains(&message.chat) {
                    info!("Stopping bot in chat: {}", message.chat.id());
                    inner.active_chats.remove_item(&message.chat);
                    inner.api.spawn(message.text_reply("Stopping bot in this chat!"));
                } else {
                    inner.api.spawn(message.text_reply("Bot is not running in this chat!"));
                }
            }

            Ok(())
        }))
    }

    fn cmd_unknown<'a>(&self, message: Message, command: String) -> BotFuture<'a> {
        let inner_arc = self.inner.clone();

        Box::new(lazy(move || {
            let inner = inner_arc.lock().unwrap();

            warn!("Unknown command {}!", command);
            inner.api.spawn(message.text_reply(format!("Invalid command `{}`!", command)));

            Ok(())
        }))
    }
}

impl InnerTelegramBot {
    fn check_admin(&self, message: &Message) -> bool {
        let authorized = match message.chat {
            Private(_) => true,
            _ => {
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

    fn send_gif<'a>(&self, reply_chat: Option<MessageChat>) -> Result<(), GabelnError> {
        let url = self.giphy.get_gif()?;

        if let Some(chat) = reply_chat {
            self.api.spawn(chat.document_url(url));
        } else {
            for chat in self.active_chats.iter() {
                self.api.spawn(chat.document_url(url.clone()));
            }
        }

        Ok(())
    }

    fn command<'a>(&self, content: &'a str, chat: &MessageChat) -> Option<&'a str> {
        debug!("Parsing command {}", content);
        if let Some(captures) = self.command_re.captures(content) {
            if let Some(receiver) = captures.get(2) {
                if let Some(ref username) = self.me.username {
                    if receiver.as_str() == username.as_str() {
                        return captures.get(1).map(|c| c.as_str());
                    }
                }
            } else if let Private(_) = chat {
                return captures.get(1).map(|c| c.as_str());
            }
        }

        None
    }
}
