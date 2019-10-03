use crate::api;
mod manga_by_title;
mod mdlist;

use scraper::{Html, Selector};

lazy_static! {
    pub static ref MDLIST: reqwest::Url = api::BASE.join("/list/").unwrap();
    pub static ref MANGA_BY_TITLE: reqwest::Url = api::BASE.join("/quick_search/").unwrap();
}

#[derive(Debug, Clone)]
pub struct ScrapManga {
    pub id: u64,
    pub title: String,
}

impl ScrapManga {
    pub fn scrap(html: &Html) -> Vec<ScrapManga> {
        html.select(&Selector::parse(".manga-entry").unwrap())
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

                ScrapManga { id, title }
            })
            .collect()
    }
}

pub use manga_by_title::*;
pub use mdlist::*;
