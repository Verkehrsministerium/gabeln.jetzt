use rocket::{State, Request, response::content};
use maud::{html, DOCTYPE, Markup};
use chrono::Utc;
use chrono_humanize::HumanTime;
use event_manager::EventManager;
use std::sync::{Arc, Mutex};

pub fn gabeln(title: &str, content: Markup) -> content::Html<String> {
    content::Html((html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (title) }
                link rel="stylesheet" type="text/css" href="semantic.min.css";
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
pub fn not_found(_req: &Request) -> content::Html<String> {
    debug!("Handling 404 request");
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
pub fn index(event_manager: State<Arc<Mutex<EventManager>>>) -> content::Html<String> {
    debug!("Handling / request");
    gabeln("gabeln.jetzt", html! {
        div.ui.feed {
            @for ref event in event_manager.inner().lock().unwrap().events.iter().rev() {
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
pub fn feed(event_manager: State<Arc<Mutex<EventManager>>>) -> content::Xml<String> {
    debug!("Handling /atom.xml request");
    content::Xml(event_manager.inner().lock().unwrap().feed.to_string())
}

#[get("/about")]
pub fn about() -> content::Html<String> {
    debug!("Handling /about request");
    gabeln("About", html! {
        h1 { "About" }
        p {
            "This site contains a feed for forks of github repositories by a couple"
            "of users."
        }
    })
}
