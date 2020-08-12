use crate::api;

#[derive(Debug)]
pub struct ChapterShort {
    pub id: u64,
    pub chapter: String,
    pub volume: String,
    pub title: String,
    pub lang_code: String
}

impl ChapterShort {
    pub fn new(id: u64, chapter_field: api::ChapterField) -> ChapterShort {
        ChapterShort {
            id,
            chapter: chapter_field.chapter,
            volume: chapter_field.volume,
            title: chapter_field.title,
            lang_code: chapter_field.lang_code
        }
    }
}

impl std::string::ToString for ChapterShort {
    fn to_string(&self) -> String {
        if self.title == "Oneshot" {
            format!("Oneshot [{}]", self.id)
        }
        else {
            match (self.title.is_empty(), self.volume.is_empty()) {
                (true, true) => sanitize_filename::sanitize(format!("Ch. {} [{}]", self.chapter, self.id)),
                (true, false) => sanitize_filename::sanitize(format!("Vol. {} Ch. {} [{}]", self.volume, self.chapter, self.id)),
                (false, true) => sanitize_filename::sanitize(format!("Ch. {} - {} [{}]", self.chapter, self.title, self.id)),
                _ => sanitize_filename::sanitize(format!(
                    "Vol. {} Ch. {} - {} [{}]",
                    self.volume, self.chapter, self.title, self.id
                )),
            }
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
}

impl std::string::ToString for Manga {
    fn to_string(&self) -> String {
        sanitize_filename::sanitize(format!("{} [{}]", self.title, self.id))
    }
}