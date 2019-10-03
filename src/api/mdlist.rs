use crate::api;

use scraper::{Html, Selector};
use std::error::Error;

pub enum Status {
    Reading,
    Completed,
    OnHold,
    PlanToRead,
    Dropped,
    ReReading,
}

#[derive(Debug, Clone)]
pub struct MDListEntry {
    pub id: u64,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct MDList {
    pub id: u64,
    pub entries: Vec<MDListEntry>,
}

impl MDList {
    pub fn scrap(client: &reqwest::Client, id: u64) -> Result<MDList, Box<dyn Error>> {
        let body = client
            .get(api::SCRAP_MDLIST.join(&id.to_string()).unwrap())
            .send()?
            .text()?;

        Ok(MDList {
            id,
            entries: Html::parse_document(&body)
                .select(&Selector::parse(".manga-entry").unwrap())
                .into_iter()
                .map(|entry_node| {
                    let element = &entry_node.value();

                    let id = element.attr("data-id").unwrap().parse::<u64>().unwrap();
                    let title: String = entry_node
                        .select(&Selector::parse(".manga_title").unwrap())
                        .into_iter()
                        .map(|title_node| title_node.value().attr("title").unwrap())
                        .collect::<Vec<&str>>()
                        .first()
                        .unwrap()
                        .to_string();

                    MDListEntry { id, title }
                })
                .collect(),
        })
    }
}
