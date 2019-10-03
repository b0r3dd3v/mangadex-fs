use crate::api::scrap;
use crate::api::scrap::ScrapManga;
use crate::api::MangadexSession;

use scraper::Html;

use std::error::Error;

pub struct MangaByTitle(pub u64);

impl MangaByTitle {
    pub fn scrap<S: AsRef<str>>(
        client: &reqwest::Client,
        session: &MangadexSession,
        title: S,
        exact: bool,
    ) -> Result<MangaByTitle, Box<dyn Error>> {
        let url = scrap::MANGA_BY_TITLE.join(title.as_ref()).unwrap();

        let body = client
            .get(url)
            .header(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_static("urmomgay"),
            )
            .header(
                reqwest::header::COOKIE,
                reqwest::header::HeaderValue::from_str(&format!("mangadex_session={}", session.id))
                    .unwrap(),
            )
            .send()?
            .text()?;

        let found = ScrapManga::scrap(&Html::parse_document(&body));

        found
            .iter()
            .find(|m| m.title == title.as_ref())
            .or(if exact { None } else { found.iter().nth(0) })
            .map(|m| MangaByTitle(m.id))
            .ok_or(format!("Manga of title \"{}\" not found.", title.as_ref()).into())
    }
}
