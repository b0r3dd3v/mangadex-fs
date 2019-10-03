mod chapter;
mod manga;
mod scrap;
mod session;

lazy_static! {
    pub static ref BASE: reqwest::Url = reqwest::Url::parse("https://mangadex.org/").unwrap();
    pub static ref MANGA: reqwest::Url = BASE.join("/api/manga/").unwrap();
    pub static ref CHAPTER: reqwest::Url = BASE.join("/api/chapter/").unwrap();
}

pub use self::chapter::ChapterResponse;
pub use self::manga::MangaResponse;
pub use self::scrap::{MDList, MangaByTitle};
pub use self::session::*;
