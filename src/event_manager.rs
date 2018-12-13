use events::{Event, EventCollector};
use error::GabelnError;
use feed;
use std::env;
use futures::{Future, Sink};
use futures::sync::mpsc::{Sender, Receiver, channel};
use chrono::{Utc};

pub struct EventManager {
    pub events: Vec<Event>,
    pub feed: String,
    sender: Sender<Event>,
}

impl EventManager {
    pub fn new() -> (Self, Receiver<Event>) {
        let events = EventCollector::default().collect().unwrap();
        let (sender, recv) = channel(100);

        (
            Self {
                feed: feed::create_feed(&events).unwrap().to_string(),
                events: events,
                sender: sender,
            },
            recv,
        )
    }

    pub fn update(&mut self) -> Result<(), GabelnError> {
        std::thread::sleep(std::time::Duration::from_secs(5));

        info!("Updating event list");
        self.events = EventCollector::default()
            .add_users(
                env::var("USERS")
                    .unwrap_or("fin-ger,jwuensche".to_string())
                    .split(",")
                    .collect::<Vec<&str>>()
            )
            .collect()?;
        self.feed = feed::create_feed(&self.events)?.to_string();

        let now = Utc::now();
        let duration = chrono::Duration::minutes(5000000);
        for event in self.events.iter() {
            if now - event.created_at < duration {
                info!("Publishing new fork event: {}", event.payload.forkee.clone().unwrap().full_name);
                self.sender
                    .clone()
                    .send(event.clone())
                    .wait()
                    .map_err(|_| GabelnError::FailedToPublishEvents)?;
            }
        }

        Ok(())
    }
}
