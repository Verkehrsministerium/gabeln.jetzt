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

mod error;
mod events;
mod feed;
mod content;

use events::EventCollector;
use rocket_contrib::serve::StaticFiles;
use std::env;

fn main() {
    let events = EventCollector::default()
        .add_users(
            env::var("USERS")
                .unwrap_or("fin-ger,jwuensche".to_string())
                .split(",")
                .collect::<Vec<&str>>()
        )
        .collect()
        .unwrap();

    let feed = feed::create_feed(&events)
        .unwrap()
        .to_string();

    // TODO: update events and feed to fetch new events
    // TODO: telegram bot

    rocket::ignite()
        .register(catchers![content::not_found])
        .mount("/", routes![content::index, content::feed, content::about])
        .mount("/assets", StaticFiles::from("assets"))
        .manage(feed)
        .manage(events)
        .launch();
}
