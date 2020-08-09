use crate::api;

#[derive(Debug)]
pub enum TagMode {
    All, Any
}

#[derive(Debug, Clone, Copy)]
pub enum Language {
    Japanese,
    English,
    Polish,
    Russian,
    German,
    French,
    Vietnamese,
    Swedish,
    Chinese,
    Indonesian,
    Korean,
    Spanish,
    Thai,
    Filipino,
    ChineseTrad
}

#[derive(Debug)]
pub struct Demographic {
    pub shounen: bool,
    pub shoujo: bool,
    pub seinen: bool,
    pub josei: bool
}

impl Default for Demographic {
    fn default() -> Self {
        Demographic {
            shounen: true,
            shoujo: true,
            seinen: true,
            josei: true
        }
    }
}

#[derive(Debug)]
pub struct PublicationStatus {
    pub ongoing: bool,
    pub completed: bool,
    pub cancelled: bool,
    pub hiatus: bool
}

impl Default for PublicationStatus {
    fn default() -> Self {
        PublicationStatus {
            ongoing: true,
            completed: true,
            cancelled: true,
            hiatus: true
        }
    }
}

impl std::convert::TryFrom<&str> for Language {
    type Error = ();

    fn try_from(string: &str) -> Result<Language, Self::Error> {
        match string {
            "japanese" | "jp" => Ok(Language::Japanese),
            "english" | "en" | "eng" | "gb" => Ok(Language::English),
            "polish" | "pol" | "pl" => Ok(Language::Polish),
            "russian" | "rus" | "ru" => Ok(Language::Russian),
            "german" | "ger" | "deu" | "de" => Ok(Language::German),
            "french" | "fre" | "fra" => Ok(Language::French),
            "vietnamese" | "vie" | "vi" => Ok(Language::Vietnamese),
            "swedish" | "swe" | "sv" => Ok(Language::Swedish),
            "chinese" | "chi" | "zho" | "zh" => Ok(Language::Chinese),
            "indonesian" | "ind" | "id" => Ok(Language::Indonesian),
            "korean" | "kor" | "ko" => Ok(Language::Korean),
            "spanish" | "spa" | "es" => Ok(Language::Spanish),
            "thai" | "tha" | "th" => Ok(Language::Thai),
            "filipino" | "fil" => Ok(Language::Filipino),
            "chinese traditional" => Ok(Language::ChineseTrad), 
            _ => Err(())
        }
    }
}

impl Language {
    pub fn code(&self) -> u8 {
        match self {
            Language::Japanese => 2u8,
            Language::English => 1u8,
            Language::Polish => 3u8,
            Language::Russian => 7u8,
            Language::German => 8u8,
            Language::French => 10u8,
            Language::Vietnamese => 12u8,
            Language::Swedish => 18u8,
            Language::Chinese => 21u8,
            Language::Indonesian => 27u8,
            Language::Korean => 28u8,
            Language::Spanish => 29u8,
            Language::Thai => 32u8,
            Language::Filipino => 34u8,
            Language::ChineseTrad => 35u8
        }
    }

    pub fn from_code(n: u8) -> Option<Self> {
        match n {
            2u8 => Some(Language::Japanese),
            1u8 => Some(Language::English),
            3u8 => Some(Language::Polish),
            7u8 => Some(Language::Russian),
            8u8 => Some(Language::German),
            10u8 => Some(Language::French),
            12u8 => Some(Language::Vietnamese),
            18u8 => Some(Language::Swedish),
            21u8 => Some(Language::Chinese),
            27u8 => Some(Language::Indonesian),
            28u8 => Some(Language::Korean),
            29u8 => Some(Language::Spanish),
            32u8 => Some(Language::Thai),
            34u8 => Some(Language::Filipino),
            35u8 => Some(Language::ChineseTrad),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct SearchParams {
    pub title: String,
    pub author: Option<String>,
    pub artist: Option<String>,
    pub original_language: Option<Language>,
    pub demographic: Demographic,
    pub publication: PublicationStatus,
    pub include_tag: Vec<api::Genre>,
    pub exclude_tag: Vec<api::Genre>,
    pub inclusion_mode: TagMode,
    pub exclusion_mode: TagMode,
}

impl Default for SearchParams {
    fn default() -> SearchParams {
        SearchParams {
            title: String::default(),
            author: None,
            artist: None,
            original_language: None,
            demographic: Demographic::default(),
            publication: PublicationStatus::default(),
            include_tag: vec![],
            exclude_tag: vec![],
            inclusion_mode: TagMode::All,
            exclusion_mode: TagMode::Any
        }
    }
}

#[derive(Debug)]
pub struct SearchEntry {
    pub id: u64,
    pub title: String
}

fn headers(session: &api::MangaDexSession) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.append(
        reqwest::header::USER_AGENT,
        api::user_agent()
    );
    headers.append(
        reqwest::header::COOKIE,
        reqwest::header::HeaderValue::from_str(&format!("mangadex_session={}", session.id))
            .unwrap()
    );

    headers
}

pub async fn search(client: &reqwest::Client, session: &api::MangaDexSession, params: &SearchParams) -> Result<Vec<SearchEntry>, reqwest::Error> {
    let mut url = reqwest::Url::parse("https://mangadex.org/search/").unwrap();

    url.query_pairs_mut().append_pair("title", &params.title);

    if let Some(ref author) = params.author { url.query_pairs_mut().append_pair("author", author); }
    if let Some(ref artist) = params.artist { url.query_pairs_mut().append_pair("artist", artist); }
    if let Some(ref language) = params.original_language { url.query_pairs_mut().append_pair("lang_id", &language.code().to_string()); }

    match params.demographic {
        Demographic { shounen, shoujo, seinen, josei } => {
            if !(shounen && shoujo && seinen && josei) || !(shounen || shoujo || seinen || josei) {
                let mut demos: Vec<&str> = vec![];

                if shounen { demos.push("1"); }
                if shoujo { demos.push("2"); }
                if seinen { demos.push("3"); }
                if josei { demos.push("4"); }

                url.query_pairs_mut().append_pair("demos", &demos.into_iter().map(String::from).collect::<Vec<_>>().join(","));
            }
        }
    };

    match params.publication {
        PublicationStatus { ongoing, completed, cancelled, hiatus } => {
            if !(ongoing && completed && cancelled && hiatus) || !(ongoing || completed || cancelled || hiatus) {
                let mut demos: Vec<&str> = vec![];

                if ongoing { demos.push("1"); }
                if completed { demos.push("2"); }
                if cancelled { demos.push("3"); }
                if hiatus { demos.push("4"); }

                url.query_pairs_mut().append_pair("statuses", &demos.into_iter().map(String::from).collect::<Vec<_>>().join(","));
            }
        }
    };

    match params.inclusion_mode {
        TagMode::All => url.query_pairs_mut().append_pair("tag_mode_inc", "all"),
        TagMode::Any => url.query_pairs_mut().append_pair("tag_mode_inc", "any"),
    };

    match params.exclusion_mode {
        TagMode::All => url.query_pairs_mut().append_pair("tag_mode_exc", "all"),
        TagMode::Any => url.query_pairs_mut().append_pair("tag_mode_exc", "any"),
    };

    if params.include_tag.len() > 0 || params.exclude_tag.len() > 0 {
        let mut tags: Vec<i8> = vec![];

        for tag in &params.include_tag {
            tags.push((tag.clone() as u8) as i8);
        }

        for tag in &params.exclude_tag {
            tags.push(-((tag.clone() as u8) as i8));
        }

        url.query_pairs_mut().append_pair("tags", &tags.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(","));
    }


    let text = client
        .get(url)
        .headers(headers(&session))
        .send().await?
        .text().await?;

    let html = scraper::Html::parse_document(text.as_str());

    Ok(html.select(&scraper::Selector::parse("div > div > div.manga-entry").unwrap())
        .into_iter()
        .map(|entry_node| {
            let element = &entry_node.value();

            let id = element.attr("data-id").unwrap().parse::<u64>().unwrap();
            let title: String = entry_node
                .select(&scraper::Selector::parse("div > a.manga_title").unwrap())
                .into_iter()
                .map(|title_node| title_node.value().attr("title").unwrap())
                .collect::<Vec<&str>>()
                .first()
                .unwrap()
                .to_string();

            SearchEntry { id, title }
        })
        .collect())
}