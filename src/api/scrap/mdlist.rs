use crate::api::scrap;
use crate::api::scrap::ScrapManga;

use std::error::Error;

use scraper::Html;

#[allow(dead_code)]
pub enum Status {
    Reading,
    Completed,
    OnHold,
    PlanToRead,
    Dropped,
    ReReading,
}

#[derive(Debug, Clone)]
pub struct MDList {
    pub id: u64,
    pub entries: Vec<scrap::ScrapManga>,
}

impl MDList {
    pub fn scrap(client: &reqwest::Client, id: u64) -> Result<MDList, Box<dyn Error>> {
        let body = client
            .get(scrap::MDLIST.join(&id.to_string()).unwrap())
            .send()?
            .text()?;

        Ok(MDList {
            id,
            entries: ScrapManga::scrap(&Html::parse_document(&body)),
        })
    }
}
