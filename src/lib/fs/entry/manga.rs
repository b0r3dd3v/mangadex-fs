use crate::api;

#[derive(Debug)]
pub struct ChapterShort {
    pub id: u64,
    pub chapter: String,
    pub volume: String,
    pub title: String,
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
}

#[derive(Debug)]
pub struct Manga {
    pub id: u64,
    pub title: String,
    pub cover: Option<reqwest::Url>,
    pub chapters: Vec<ChapterShort>,
    
    pub time: time::Timespec,
    pub uid: nix::unistd::Uid,
    pub gid: nix::unistd::Gid
}

impl Manga {
    pub fn new(id: u64, time: time::Timespec, uid: nix::unistd::Uid, gid: nix::unistd::Gid, manga_api: api::Manga) -> Manga {
        Manga {
            id,
            title: manga_api.manga.title,
            cover: reqwest::Url::parse("https://mangadex.org/").unwrap().join(&manga_api.manga.cover_url).ok(),
            chapters: manga_api.chapter
                .into_iter()
                .map(|(id, chapter)| ChapterShort::new(id, chapter))
                .collect(),

            time,
            uid,
            gid
        }
    }
}