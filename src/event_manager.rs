use events::{Event, EventCollector};
use error::GabelnError;
use feed;
use std::env;

pub struct EventManager {
    pub events: Vec<Event>,
    pub feed: String,
}

impl Default for EventManager {
    fn default() -> Self {
        let events = EventCollector::default().collect().unwrap();

        Self {
            feed: feed::create_feed(&events).unwrap().to_string(),
            events: events,
        }
    }
}

impl EventManager {
    pub fn update(&mut self) -> Result<(), GabelnError> {
        self.events = EventCollector::default()
            .add_users(
                env::var("USERS")
                    .unwrap_or("fin-ger,jwuensche".to_string())
                    .split(",")
                    .collect::<Vec<&str>>()
            )
            .collect()?;
        self.feed = feed::create_feed(&self.events)?.to_string();

        Ok(())
    }
}
