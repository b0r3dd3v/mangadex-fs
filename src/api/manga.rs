use crate::api;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct MangaField {
    pub title: String,
    pub cover_url: String,
    pub lang_name: String,
    pub lang_flag: String,
    pub genres: Vec<u8>,
    pub description: String,
    pub artist: String,
    pub author: String,
    pub status: u8,
    pub last_chapter: String,
    pub hentai: u8,
    pub links: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ChapterField {
    pub chapter: String,
    pub volume: String,
    pub title: String,
    pub lang_code: String,
    pub timestamp: u32,
}

#[derive(Debug, Deserialize)]
pub struct MangaResponse {
    pub manga: MangaField,
    pub chapter: HashMap<u64, ChapterField>,
    pub status: String,
}

impl MangaResponse {
    pub fn get(client: &reqwest::Client, id: u64) -> Result<MangaResponse, reqwest::Error> {
        client
            .get(api::MANGA.join(&id.to_string()).unwrap())
            .send()?
            .json()
    }
}
