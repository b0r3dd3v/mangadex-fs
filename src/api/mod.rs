mod chapter;
mod manga;

lazy_static! {
    pub static ref BASE: reqwest::Url = reqwest::Url::parse("https://mangadex.org/").unwrap();
    pub static ref API_MANGA: reqwest::Url = BASE.join("/api/manga/").unwrap();
    pub static ref API_CHAPTER: reqwest::Url = BASE.join("/api/chapter/").unwrap();
}

pub use self::chapter::ChapterResponse;
pub use self::manga::MangaResponse;