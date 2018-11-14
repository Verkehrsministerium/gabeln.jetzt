#![feature(proc_macro_hygiene, decl_macro)]

extern crate regex;
extern crate reqwest;
extern crate chrono;
extern crate rayon;
extern crate atom_syndication;
#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate serde;

mod error;
mod events;
mod feed;

use events::EventCollector;
use rocket::{State, Request};

#[catch(404)]
fn not_found(_req: &Request) -> &'static str {
    "Weeeeeeee! You reached the end of the internet."
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/atom.xml", format = "application/atom+xml")]
fn feed(feed: State<String>) -> String {
    feed.inner().to_string()
}

fn main() {
    let events = EventCollector::default()
        .add_user("jwuensche")
        .add_user("fin-ger")
        .add_users(vec!["johannwagner", "martin31821"])
        .collect()
        .unwrap();

    let feed = feed::create_feed(&events)
        .unwrap()
        .to_string();

    // TODO: update events and feed to fetch new events

    rocket::ignite()
        .register(catchers![not_found])
        .mount("/", routes![index, feed])
        .manage(feed)
        .launch();
}
