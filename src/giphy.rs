use rand::{thread_rng, seq::SliceRandom};
use reqwest::Client;
use std::env;
use error::GabelnError;

pub struct Giphy {
    client: Client,
    api_key: String,
    gif_limit: String,
}

#[derive(Deserialize, Clone)]
pub struct GiphyOriginal {
    pub url: String,
    pub width: String,
    pub height: String,
    pub size: String,
    pub frames: String,
}

#[derive(Deserialize, Clone)]
pub struct GiphyImages {
    pub original: GiphyOriginal,
}

#[derive(Deserialize, Clone)]
pub struct GiphyGif {
    pub images: GiphyImages,
}

#[derive(Deserialize, Clone)]
pub struct GiphyResponse {
    pub data: Vec<GiphyGif>,
}

impl Giphy {
    pub fn new() -> Result<Self, GabelnError> {
        Ok(Self {
            client: Client::new(),
            api_key: env::var("GIPHY_API_KEY")
                .map_err(|_| GabelnError::NoGiphyApiKey)?,
            gif_limit: env::var("GIPHY_GIF_LIMIT")
                .unwrap_or_else(|_| "30".to_string()),
        })
    }

    pub fn get_gif(&self) -> Result<String, GabelnError> {
        let url = format!(
            "https://api.giphy.com/v1/gifs/search?api_key={}&q=fork+food&limit={}",
            self.api_key,
            self.gif_limit,
        );
        let mut rng = thread_rng();
        debug!("Fetching Giphy API: {}", url);

        let mut response = self.client
            .get(&url)
            .send()
            .map_err(|e| {
                error!("Failed to fetch gif from giphy: {}", e);
                GabelnError::FailedToFetchGif
            })?;

        let result = response
            .json::<GiphyResponse>()
            .map_err(|_| GabelnError::FailedToParseGiphyResponse)?;

        debug!("Selecting gif for telegram bot...");

        match result.data.choose(&mut rng) {
            Some(item) => Ok(item.images.original.url.clone()),
            None => Err(GabelnError::FailedToParseGiphyResponse),
        }
    }
}
