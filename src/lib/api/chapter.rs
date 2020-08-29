#[derive(Debug, serde::Deserialize)]
pub struct Chapter {
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
    pub group_name: Option<String>,
    pub group_id_2: u64,
    pub group_name_2: Option<String>,
    pub group_id_3: u64,
    pub group_name_3: Option<String>,
    pub comments: Option<u64>,
    pub server: String,
    pub page_array: Vec<String>,
    pub long_strip: bool,
    pub external: Option<String>
}

impl Chapter {
    pub async fn get(client: &reqwest::Client, id: u64) -> Result<Chapter, reqwest::Error> {
        client
            .get(reqwest::Url::parse("https://mangadex.org/api/chapter/").unwrap().join(&id.to_string()).unwrap())
            .send().await?
            .json().await
    }
}