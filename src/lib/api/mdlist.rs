use crate::api;

#[derive(Debug)]
pub enum MDListStatus {
    Reading,
    Completed,
    OnHold,
    PlanToRead,
    Dropped,
    ReReading
}

impl MDListStatus {
    pub fn display(&self) -> &str {
        match self {
            MDListStatus::Reading => "Reading",
            MDListStatus::Completed => "Completed",
            MDListStatus::OnHold => "On hold",
            MDListStatus::PlanToRead => "Plan to read",
            MDListStatus::Dropped => "Dropped",
            MDListStatus::ReReading => "Re-reading"
        }
    }
}

#[derive(Debug)]
pub struct MDListEntry {
    pub id: u64,
    pub title: String,
    pub author: String,
    pub status: MDListStatus,
    pub last_update: String
}

#[derive(Debug)]
pub struct MDListParams {
    pub id: u64,
    pub sort_by: api::SortBy
}

impl Default for MDListParams {
    fn default() -> MDListParams {
        MDListParams {
            id: 0u64,
            sort_by: api::SortBy(api::SortMode::Ascending, api::SortParameter::LastUpdated)
        }
    }
}

fn headers(session: &Option<api::MangaDexSession>) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.append(
        reqwest::header::USER_AGENT,
        api::user_agent()
    );

    if let Some(session) = session {
        headers.append(
            reqwest::header::COOKIE,
            reqwest::header::HeaderValue::from_str(&format!("mangadex_session={}", session.id))
                .unwrap()
        );
    }

    headers.append(
        reqwest::header::COOKIE,
        reqwest::header::HeaderValue::from_str(&format!("mangadex_title_mode={}", "2"))
            .unwrap()
    );

    headers
}

pub async fn mdlist(client: &reqwest::Client, session: &Option<api::MangaDexSession>, params: &MDListParams) -> Result<Vec<MDListEntry>, reqwest::Error> {    
    let mut url = reqwest::Url::parse("https://mangadex.org/list/").unwrap().join(&params.id.to_string()).unwrap();

    url.query_pairs_mut().append_pair("s", params.sort_by.encode().to_string().as_str());

    let text = client
        .get(url)
        .headers(headers(session))
        .send().await?
        .text().await?;

        use tokio::io::AsyncWriteExt;
    let mut dump = tokio::fs::File::create("dump").await.unwrap();
    dump.write_all(text.as_bytes()).await.ok();

    let html = scraper::Html::parse_document(text.as_str());

    Ok(html.select(&scraper::Selector::parse("div#content > div.manga-entry").unwrap())
        .into_iter()
        .map(|entry_node| {
            let element = &entry_node.value();

            let id = element.attr("data-id").unwrap().parse::<u64>().unwrap();
            let row_selector = scraper::Selector::parse("div > div.row > div").unwrap();
            let mut rows = entry_node.select(&row_selector);

            let link_selector = scraper::Selector::parse("a").unwrap();

            let title = rows
                .nth(0)
                .and_then(|el| el.select(&link_selector)
                    .next()
                    .and_then(|el| el.value().attr("title"))
                ).unwrap_or("<unknown title>");
            let author = rows
                .nth(1)
                .and_then(|el| el.select(&link_selector)
                    .next()
                    .and_then(|el| el.value().attr("title"))
                ).unwrap_or("<unknown author>");
            let status = rows
                .nth(0)
                .and_then(|el| {
                    el.select(&scraper::Selector::parse("button").unwrap())
                        .into_iter()
                        .next()
                        .and_then(|button| button.value().attr("title"))
                })
                .and_then(|string: &str| match string {
                    "Reading" => Some(api::MDListStatus::Reading),
                    "Completed" => Some(api::MDListStatus::Completed),
                    "On hold" => Some(api::MDListStatus::OnHold),
                    "Plan to read" => Some(api::MDListStatus::PlanToRead),
                    "Dropped" => Some(api::MDListStatus::Dropped),
                    "Re-reading" => Some(api::MDListStatus::ReReading),
                    _ => None
                }).unwrap_or(api::MDListStatus::PlanToRead);

            let last_update = rows
            .last()
            .map(|el| el
                .text()
                .fold(String::from(""), |acc, text| acc + text))
            .unwrap_or(String::from("-")).trim().to_string();

            MDListEntry { id, title: title.to_string(), author: author.to_string(), status, last_update }
        })
        .collect())
}