mod chapter;
mod manga;
mod mdlist;

lazy_static! {
    pub static ref BASE: reqwest::Url = reqwest::Url::parse("https://mangadex.org/").unwrap();
    pub static ref API_MANGA: reqwest::Url = BASE.join("/api/manga/").unwrap();
    pub static ref API_CHAPTER: reqwest::Url = BASE.join("/api/chapter/").unwrap();
    pub static ref SCRAP_MDLIST: reqwest::Url = BASE.join("/list/").unwrap();
}

pub use self::chapter::ChapterResponse;
pub use self::manga::MangaResponse;
pub use self::mdlist::*;
