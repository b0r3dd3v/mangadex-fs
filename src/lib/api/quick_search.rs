use crate::api;

#[derive(Debug)]
pub struct QuickSearchEntry {
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

pub async fn quick_search<S: AsRef<str>>(client: &reqwest::Client, session: &api::MangaDexSession, query: S) -> Result<Vec<QuickSearchEntry>, reqwest::Error> {
    let url = reqwest::Url::parse("https://mangadex.org/quick_search/").unwrap().join(query.as_ref()).unwrap();

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

            QuickSearchEntry { id, title }
        })
        .collect())
}