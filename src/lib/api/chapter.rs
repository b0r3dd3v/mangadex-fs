fn deserialize_long_strip_flag<'de, D>(deserializer: D) -> Result<bool, D::Error>
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
    #[serde(deserialize_with = "deserialize_long_strip_flag")]
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