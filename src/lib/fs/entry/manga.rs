use crate::api;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct ChapterShort {
    pub id: u64,
    pub chapter: String,
    pub volume: String,
    pub title: String,
}

impl std::hash::Hash for ChapterShort {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl ChapterShort {
    pub fn new(id: u64, chapter_field: api::ChapterField) -> ChapterShort {
        ChapterShort {
            id,
            chapter: chapter_field.chapter,
            volume: chapter_field.volume,
            title: chapter_field.title
        }
    }

    pub fn display(&self) -> String {
        let hash = {
            let mut s = std::collections::hash_map::DefaultHasher::new();
            self.hash(&mut s);
            s.finish()
        };

        match (self.title.is_empty(), self.volume.is_empty()) {
            (true, true) => sanitize_filename::sanitize(format!("{} [{:06x}]", self.chapter, hash)),
            (true, false) => sanitize_filename::sanitize(format!("{}.{} [{:06x}]", self.volume, self.chapter, hash)),
            (false, true) => sanitize_filename::sanitize(format!("{} {} [{:06x}]", self.chapter, self.title, hash)),
            _ => sanitize_filename::sanitize(format!(
                "{}.{} {} [{:06x}]",
                self.volume, self.chapter, self.title, hash
            )),
        }
    }
}

#[derive(Debug)]
pub struct Manga {
    pub id: u64,
    pub title: String,
    pub cover: Option<reqwest::Url>,
    pub chapters: Vec<ChapterShort>,
}

impl Hash for Manga {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Manga {
    pub fn new(id: u64, manga_api: api::Manga) -> Manga {
        Manga {
            id,
            title: manga_api.manga.title,
            cover: reqwest::Url::parse("https://mangadex.org/").unwrap().join(&manga_api.manga.cover_url).ok(),
            chapters: manga_api.chapter
                .into_iter()
                .map(|(id, chapter)| ChapterShort::new(id, chapter))
                .collect()
        }
    }

    pub fn display(&self) -> String {
        let hash = {
            let mut s = std::collections::hash_map::DefaultHasher::new();
            self.hash(&mut s);
            s.finish()
        };

        sanitize_filename::sanitize(format!("{} [{:06x}]", self.title, hash))
    }
}