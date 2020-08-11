use crate::api;

#[derive(Debug)]
pub struct FollowsEntry {
    pub manga_id: u64,
    pub manga_title: String,
    pub chapter_id: u64,
    pub chapter: String,
    pub chapter_title: String,
    pub chapter_volume: String,
    pub marked_read: bool,
    pub last_update: String
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

pub async fn follows(client: &reqwest::Client, session: &api::MangaDexSession) -> Result<Vec<FollowsEntry>, reqwest::Error> {    
    let url = reqwest::Url::parse("https://mangadex.org/follows/").unwrap();

    let text = client
        .get(url)
        .headers(headers(session))
        .send().await?
        .text().await?;
        
    let html = scraper::Html::parse_document(text.as_str());

    let row_selector = scraper::Selector::parse("div#chapters > div.chapter-container > div.row").unwrap();

    let mut rows = html.select(&row_selector);

    rows.next();

    let mut previous_manga_title = String::from("<unknown title>");

    Ok(rows.into_iter()
        .map(|row| {
            let link_selector = scraper::Selector::parse("div > a.manga_title").unwrap();

            let manga_title = row.select(&link_selector)
                .into_iter()
                .map(|link| link.value().attr("title").unwrap())
                .collect::<Vec<_>>()
                .first()
                .unwrap_or(&previous_manga_title.as_str())
                .to_string();

            previous_manga_title = manga_title.clone();

            let chapter_row_selector = scraper::Selector::parse("div > div.chapter-row").unwrap();

            let mut entry: Vec<api::FollowsEntry> = row.select(&chapter_row_selector)
                .into_iter()
                .map(|chapter_row| {
                    let last_update = chapter_row.select(&scraper::Selector::parse("div").unwrap())
                        .nth(4)
                        .map(|div| div.text().fold(String::from(""), |acc, text| acc + text))
                        .unwrap()
                        .to_string();

                    let marked_read: bool = chapter_row.select(&scraper::Selector::parse("div > span").unwrap())
                        .nth(0)
                        .and_then(|span| span.value().attr("title"))
                        .map(|title| title == "Mark unread")
                        .unwrap();
                        
                    let chapter_row = chapter_row.value();

                    let manga_id = chapter_row.attr("data-manga-id").unwrap().parse::<u64>().unwrap();
                    let chapter_id = chapter_row.attr("data-id").unwrap().parse::<u64>().unwrap();
                    let chapter_title = chapter_row.attr("data-title").unwrap().to_string();
                    let chapter = chapter_row.attr("data-chapter").unwrap().to_string();
                    let chapter_volume = chapter_row.attr("data-volume").unwrap().to_string();



                    FollowsEntry {
                        manga_id: manga_id, 
                        manga_title: manga_title.clone(),
                        chapter_id: chapter_id,
                        chapter: chapter.to_string(),
                        chapter_title: chapter_title.to_string(),
                        chapter_volume: chapter_volume.to_string(),
                        marked_read: marked_read,
                        last_update: last_update.trim().to_string()
                    }
                })
                .collect::<Vec<_>>();

            entry.pop().unwrap()
        })
        .collect())
}