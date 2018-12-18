#![feature(proc_macro_hygiene, decl_macro)]
#![feature(impl_trait_in_bindings)]

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
extern crate fern;
extern crate rand;

mod error;
mod events;
mod feed;
mod content;
mod event_manager;
mod telegram;
mod giphy;

use event_manager::EventManager;
use telegram::TelegramBot;
use rocket_contrib::serve::StaticFiles;
use clokwerk::{Scheduler, TimeUnits};
use chrono::SecondsFormat;
use fern::colors::{ColoredLevelConfig, Color};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let colors_level = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Cyan)
        .trace(Color::Magenta);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_timestamp}{timestamp}{reset} [{level}] {bold}<{location}>{reset} {message}",
                bold = "\x1B[1m",
                reset = "\x1B[0m",
                color_timestamp = format!("\x1B[{}m", Color::Blue.to_fg_str()),
                timestamp = chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
                location = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("gabeln_jetzt", log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let (event_manager, recv) = EventManager::new();
    let events = Arc::new(Mutex::new(event_manager));
    let update_events = events.clone();
    let update = move || {
        match update_events.lock().unwrap().update() {
            Ok(_) => {
            },
            Err(e) => {
                error!("{:?}", e);
            },
        }
    };

    thread::spawn(move || {
        TelegramBot::new().unwrap().run(recv).unwrap();
    });

    let mut scheduler = Scheduler::new();
    scheduler.every(5.minutes()).run(update.clone());
    let _handle = scheduler.watch_thread(std::time::Duration::from_millis(500));
    thread::spawn(update);

    rocket::ignite()
        .register(catchers![content::not_found])
        .mount("/", routes![content::index, content::feed, content::about])
        .mount("/", StaticFiles::from("assets"))
        .manage(events)
        .launch();
}
