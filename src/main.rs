#![feature(proc_macro_hygiene, decl_macro)]

extern crate regex;
extern crate reqwest;
extern crate chrono;
extern crate chrono_humanize;
extern crate rayon;
extern crate atom_syndication;
extern crate maud;
#[macro_use] extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate clokwerk;
extern crate futures;
extern crate tokio;
extern crate telegram_bot_fork;
#[macro_use]
extern crate log;
#[macro_use(slog_o, slog_kv)]
extern crate slog;
extern crate slog_stdlog;
extern crate slog_scope;
extern crate slog_term;
extern crate slog_async;

mod error;
mod events;
mod feed;
mod content;
mod event_manager;
mod telegram;

use event_manager::EventManager;
use telegram::TelegramBot;
use rocket_contrib::serve::StaticFiles;
use clokwerk::{Scheduler, TimeUnits};
use slog::Drain;
use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, slog_o!("version" => env!("CARGO_PKG_VERSION")));

    let _scope_guard = slog_scope::set_global_logger(logger);
    let _log_guard = slog_stdlog::init().unwrap();

    let events = Arc::new(RwLock::new(EventManager::default()));
    let update_events = events.clone();
    let update = move || {
        match update_events.write().unwrap().update() {
            Ok(_) => {
            },
            Err(e) => {
                error!("{:?}", e);
            },
        }
    };

    // TODO: add logging
    // TODO: add channel that publishes new events (last 5 minutes)
    // TODO: use different tokio runtime
    // TODO: crawl random gif from giphy
    // TODO: send gif in telegram bot

    // TODO: remove this thread
    thread::spawn(|| {
        TelegramBot::new().unwrap().run().unwrap();
    });

    let mut scheduler = Scheduler::new();
    scheduler.every(5.minutes()).run(update.clone());
    scheduler.watch_thread(std::time::Duration::from_millis(500));
    thread::spawn(update);

    rocket::ignite()
        .register(catchers![content::not_found])
        .mount("/", routes![content::index, content::feed, content::about])
        .mount("/assets", StaticFiles::from("assets"))
        .manage(events)
        .launch();
}
