use crate::api;

fn deserialize_hentai_flag<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = bool;
    
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "1 or 0")
        }
    
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error 
        {
            Ok(v == 1u64)
        }
    }

    deserializer.deserialize_any(Visitor)
}

#[derive(Debug, serde::Deserialize)]
pub struct MangaDetails {
    pub title: String,
    pub cover_url: String,
    pub lang_name: String,
    pub lang_flag: String,
    pub genres: Vec<api::Genre>,
    pub description: String,
    pub artist: String,
    pub author: String,
    pub status: api::MangaStatus,
    pub last_chapter: String,
    #[serde(deserialize_with = "deserialize_hentai_flag")]
    pub hentai: bool,
    pub links: std::collections::HashMap<String, String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ChapterField {
    pub chapter: String,
    pub volume: String,
    pub title: String,
    pub lang_code: String,
    pub timestamp: u32,
}

#[derive(Debug, serde::Deserialize)]
pub struct Manga {
    pub manga: MangaDetails,
    pub chapter: std::collections::HashMap<u64, ChapterField>
}

impl Manga {
    pub async fn get(client: &reqwest::Client, id: u64) -> Result<Manga, reqwest::Error> {
        client
            .get(reqwest::Url::parse("https://mangadex.org/api/manga/").unwrap().join(&id.to_string()).unwrap())
            .send().await?
            .json::<Manga>().await
    }
}