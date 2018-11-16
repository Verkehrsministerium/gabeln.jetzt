use reqwest::{Client, header::AUTHORIZATION};
use regex::Regex;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use std::env;

use error::GabelnError;

#[derive(Deserialize, Clone)]
pub struct Actor {
    pub display_login: String,
    pub avatar_url: String,
}

#[derive(Deserialize, Clone)]
pub struct Repository {
    pub name: String,
}

#[derive(Deserialize, Clone)]
pub struct Forkee {
    pub full_name: String,
    pub html_url: String,
}

#[derive(Deserialize, Clone)]
pub struct Payload {
    pub forkee: Option<Forkee>,
}

#[derive(Deserialize, Clone)]
pub struct Event {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub actor: Actor,
    pub repo: Repository,
    pub payload: Payload,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EventCollector<'a> {
    client: Client,
    re: Regex,
    users: Vec<&'a str>,
    oauth_token: String,
}

impl<'a> Default for EventCollector<'a> {
    fn default() -> Self {
        Self {
            client: Client::new(),
            re: Regex::new(",?.*page=\\d+.*; rel=\"next\",?.*").unwrap(),
            users: Vec::new(),
            oauth_token: env::var("OAUTH_TOKEN").unwrap_or(String::new()),
        }
    }
}

impl<'a> EventCollector<'a> {
    pub fn add_user(mut self, user: &'a str) -> Self {
        self.users.push(user);

        self
    }

    pub fn add_users(mut self, mut users: Vec<&'a str>) -> Self {
        self.users.append(&mut users);

        self
    }

    pub fn collect(self) -> Result<Vec<Event>, GabelnError> {
        let mut events = self.users
            .par_iter()
            .map(|username| self.get_events_of_user(username))
            .collect::<Result<Vec<Vec<Event>>, GabelnError>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<Event>>();

        events.sort_unstable_by_key(|ev| ev.created_at);

        Ok(events)
    }

    fn get_events_of_user(&self, user: &str) -> Result<Vec<Event>, GabelnError> {
        let mut page: u32 = 1;
        let mut events = Vec::new();

        loop {
            let url = format!("https://api.github.com/users/{}/events/public?page={}&per_page=300", user, page);
            let mut response = self.client
                .get(&url)
                .header(AUTHORIZATION, format!("token {}", self.oauth_token))
                .send()
                .map_err(|_| GabelnError::FailedToFetchUserEvents(user.into()))?;

            {
                let links = response
                    .headers()
                    .get("Link")
                    .and_then(|l| l.to_str().ok())
                    .unwrap_or("");

                if self.re.is_match(links) {
                    page += 1;
                } else {
                    break;
                }
            }

            events.append(
                &mut response
                    .json::<Vec<Event>>()
                    .map_err(|_| GabelnError::FailedToParseUserEvents)?
                    .into_iter()
                    .filter(|event| event.event_type == "ForkEvent")
                    .collect::<Vec<Event>>()
            );
        }

        Ok(events)
    }
}
