use crate::api;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ChapterResponse {
    pub id: u64,
    pub timestamp: u64,
    pub hash: String,
    pub volume: String,
    pub chapter: String,
    pub title: String,
    pub lang_name: String,
    pub lang_code: String,
    pub manga_id: u64,
    pub group_id: u64,
    pub group_id_2: u64,
    pub group_id_3: u64,
    pub comments: Option<u64>,
    pub server: String,
    pub page_array: Vec<String>,
    pub long_strip: u64,
    pub status: String,
    pub external: Option<String>,
}

impl ChapterResponse {
    pub fn get(client: &reqwest::Client, id: u64) -> Result<ChapterResponse, reqwest::Error> {
        client
            .get(api::CHAPTER.join(&id.to_string()).unwrap())
            .send()?
            .json()
    }
}
