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
use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let events = Arc::new(RwLock::new(EventManager::default()));
    let update_events = events.clone();
    let update = move || {
        println!("Updating events...");

        match update_events.write().unwrap().update() {
            Ok(_) => {
            },
            Err(e) => {
                println!("{:?}", e);
            },
        }
    };

    // TODO: add logging

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
