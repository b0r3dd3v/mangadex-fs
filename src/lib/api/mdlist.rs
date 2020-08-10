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
pub struct MDListNotLoggedInEntry {
    pub id: u64,
    pub title: String,
    pub status: MDListStatus
}

#[derive(Debug)]
pub enum MDList {
    LoggedIn(Vec<api::MDListEntry>),
    NotLoggedIn(Vec<api::MDListNotLoggedInEntry>)
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

fn headers_public() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.append(
        reqwest::header::USER_AGENT,
        api::user_agent()
    );

    headers
}

pub async fn mdlist(client: &reqwest::Client, session: &api::MangaDexSession, list_id: u64) -> Result<Vec<MDListEntry>, reqwest::Error> {    
    api::set_view_mode(client, session, api::ViewMode::SimpleList).await.ok();

    let mut url = reqwest::Url::parse("https://mangadex.org/list/").unwrap().join(&list_id.to_string()).unwrap();

    url.query_pairs_mut().append_pair("s", "0");

    let text = client
        .get(url)
        .headers(headers(session))
        .send().await?
        .text().await?;

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
                    let span_selector = scraper::Selector::parse("span").unwrap();
                    let mut spans = el.select(&span_selector);

                    let string = spans
                        .nth(1)
                        .map(|span| span
                            .text()
                            .fold(String::from(""), |acc, text| acc + text));
                       
                    println!("string: {:?}", string);
                    string
                }).and_then(|string: String| match string.as_str() {
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

pub async fn mdlist_notloggedin(client: &reqwest::Client, list_id: u64) -> Result<Vec<MDListNotLoggedInEntry>, reqwest::Error> {    
    let mut url = reqwest::Url::parse("https://mangadex.org/list/").unwrap().join(&list_id.to_string()).unwrap();

    url.query_pairs_mut().append_pair("s", "0");

    let text = client
        .get(url)
        .headers(headers_public())
        .send().await?
        .text().await?;

    let html = scraper::Html::parse_document(text.as_str());

    Ok(html.select(&scraper::Selector::parse("div#content > div > div.manga-entry").unwrap())
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

            let status = entry_node
                .select(&scraper::Selector::parse("ul > li > button").unwrap())
                .into_iter()
                .map(|status_node| status_node.value().attr("title").unwrap())
                .collect::<Vec<&str>>()
                .first()
                .unwrap()
                .to_string();

            MDListNotLoggedInEntry { id, title, status: match status.as_str() {
                "Reading" => MDListStatus::Reading,
                "Completed" => MDListStatus::Completed,
                "On hold" => MDListStatus::OnHold,
                "Plan to read" => MDListStatus::PlanToRead,
                "Dropped" => MDListStatus::Dropped,
                "Re-reading" => MDListStatus::ReReading,
                _ => MDListStatus::PlanToRead
            } }
        })
        .collect())
}