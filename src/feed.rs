use chrono::Utc;
use atom_syndication::{Feed, FeedBuilder, PersonBuilder, LinkBuilder, EntryBuilder, ContentBuilder};

use error::GabelnError;
use events::Event;

pub fn create_feed(events: &Vec<Event>) -> Result<Feed, GabelnError> {
    let mut entries = Vec::new();

    for ref event in events.iter().rev() {
        entries.push(
            EntryBuilder::default()
                .title(format!("{} forked {}", event.actor.display_login, event.repo.name))
                .id(event.id.clone())
                .updated(event.created_at.to_rfc3339())
                .authors(vec![
                    PersonBuilder::default()
                        .name(event.actor.display_login.clone())
                        .uri(format!("https://github.com/{}", event.actor.display_login))
                        .build()
                        .map_err(|_| GabelnError::FailedToCreateFeed)?
                ])
                .links(vec![
                    LinkBuilder::default()
                        .href(event.payload.forkee.clone().unwrap().html_url)
                        .rel("related")
                        .mime_type(Some("text/html".into()))
                        .title(event.payload.forkee.clone().unwrap().full_name)
                        .build()
                        .map_err(|_| GabelnError::FailedToCreateFeed)?
                ])
                .published(event.created_at.to_rfc3339())
                .summary(format!(
                    "{} was forked by {} at {}.",
                    event.repo.name,
                    event.actor.display_login,
                    event.payload.forkee.clone().unwrap().full_name,
                ))
                .content(
                    ContentBuilder::default()
                        .value(format!(
                            "<a href=\"{}\"><img src=\"{}\"/></a>",
                            event.payload.forkee.clone().unwrap().html_url,
                            event.actor.avatar_url,
                        ))
                        .content_type(Some("text/html".into()))
                        .build()
                        .map_err(|_| GabelnError::FailedToCreateFeed)?
                )
                .build()
                .map_err(|_| GabelnError::FailedToCreateFeed)?
        );
    }

    Ok(
        FeedBuilder::default()
            .title("gabeln.jetzt")
            .id("gabeln.jetzt")
            .updated(
                events
                    .last()
                    .map_or(Utc::now(), |ev| ev.created_at)
                    .to_rfc3339()
            )
            .authors(vec![
                PersonBuilder::default()
                    .name("Fin Christensen")
                    .email(Some("fchristensen@embedded.enterprises".into()))
                    .uri(Some("https://blog.like-a-fin.lol".into()))
                    .build()
                    .map_err(|_| GabelnError::FailedToCreateFeed)?,
                PersonBuilder::default()
                    .name("Johannes Wuensche")
                    .email(Some("johannes.wuensche@st.ovgu.de".into()))
                    .uri(Some("https://github.com/jwuensche".into()))
                    .build()
                    .map_err(|_| GabelnError::FailedToCreateFeed)?,
            ])
            .icon(Some("/assets/icon.jpg".into()))
            .links(vec![
                LinkBuilder::default()
                    .href("/atom.xml")
                    .rel("self")
                    .hreflang(Some("en".into()))
                    .mime_type(Some("application/atom+xml".into()))
                    .title(Some("gabeln.jetzt".into()))
                    .build()
                    .map_err(|_| GabelnError::FailedToCreateFeed)?,
                LinkBuilder::default()
                    .href("/")
                    .rel("alternate")
                    .hreflang(Some("en".into()))
                    .mime_type(Some("text/html".into()))
                    .title(Some("gabeln.jetzt".into()))
                    .build()
                    .map_err(|_| GabelnError::FailedToCreateFeed)?,
            ])
            .logo(Some("/assets/logo.jpg".into()))
            .entries(entries)
            .subtitle(Some("GitHub Fork Feed".into()))
            .build()
            .map_err(|_| GabelnError::FailedToCreateFeed)?
    )
}
