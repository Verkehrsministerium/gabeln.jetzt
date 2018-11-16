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

use events::{Event, EventCollector};
use rocket::{State, Request, response::content};
use rocket_contrib::serve::StaticFiles;
use maud::{html, DOCTYPE, Markup};
use chrono::Utc;
use chrono_humanize::HumanTime;
use std::env;

fn gabeln(title: &str, content: Markup) -> content::Html<String> {
    content::Html((html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (title) }
                link rel="stylesheet" type="text/css" href="assets/semantic.min.css";
                script src="assets/jquery.slim.min.js" { }
                script src="assets/semantic.min.js" { }
            }

            body {
                div.ui.stackable.menu.borderless {
                    div.ui.text.container {
                        div.header.item { "gabeln.jetzt" }
                        a.item href="/" { "Home" }
                        a.item href="/atom.xml" { "Atom Feed" }
                        a.item href="/about" { "About" }
                    }
                }
                div.ui.text.container {
                    (content)
                    p.ui.basic.padded.center.aligned.segment {
                        "gabeln.jetzt is powered by "
                        a href="https://rocket.rs" { "rocket" }
                        " science, "
                        a href="https://semantic-ui.com" { "Semantic UI" }
                        ", the "
                        a href="https://developer.github.com/v3/" { "GitHub API" }
                        ", and the "
                        a href="https://developers.giphy.com/" { "Giphy API" }
                        "."
                    }
                }
            }
        }
    }).into_string())
}

#[catch(404)]
fn not_found(_req: &Request) -> content::Html<String> {
    gabeln("Not found", html! {
        div.ui.placeholder.segment {
            div.ui.icon.header {
                i.minus.circle.icon style="margin: 0.25em" { }
                "Weeeeeeee!"
                p style="font-weight: normal; font-size: 75%" { "You reached the end of the internet." }
            }
            a.ui.primary.button href="/" { "Go back" }
        }
    })
}

#[get("/")]
fn index(events: State<Vec<Event>>) -> content::Html<String> {
    gabeln("gabeln.jetzt", html! {
        div.ui.feed {
            @for ref event in events.inner().iter().rev() {
                div.event {
                    div.label {
                        a href=(format!("https://github.com/{}", event.actor.display_login)) {
                            img src=(event.actor.avatar_url);
                        }
                    }
                    div.content style="margin-bottom: 2em" {
                        div.date {
                            (HumanTime::from(event.created_at - Utc::now()))
                        }
                        div.summary {
                            (event.actor.display_login)
                            " forked "
                            a href=(format!("https://github.com/{}", event.repo.name)) {
                                (event.repo.name)
                            }
                            " at "
                            a href=(event.payload.forkee.clone().unwrap().html_url) {
                                (event.payload.forkee.clone().unwrap().full_name)
                            }
                            "!"
                        }
                    }
                }
            }
        }
    })
}

#[get("/atom.xml")]
fn feed(feed: State<String>) -> content::Xml<String> {
    content::Xml(feed.inner().to_string())
}

#[get("/about")]
fn about() -> content::Html<String> {
    gabeln("About", html! {
        h1 { "About" }
        p {
            "This site contains a feed for forks of github repositories by a couple"
            "of users."
        }
    })
}

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
        .register(catchers![not_found])
        .mount("/", routes![index, feed, about])
        .mount("/assets", StaticFiles::from("assets"))
        .manage(feed)
        .manage(events)
        .launch();
}
