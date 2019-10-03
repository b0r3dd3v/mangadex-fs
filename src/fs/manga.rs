use chrono;

use crate::api;
use crate::fs::chapter_info::ChapterInfo;
use crate::fs::entry::{Entry, GID, UID};

use sanitize_filename::sanitize;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct MangaEntry {
    pub id: u64,
    pub title: String,
    pub cover: reqwest::Url,
    pub chapters: Vec<ChapterInfo>,
    pub time: time::Timespec,
    pub uid: UID,
    pub gid: GID,
}

impl MangaEntry {
    pub fn get_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
}

impl Hash for MangaEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl MangaEntry {
    pub fn format(&self) -> String {
        sanitize(format!("{} [{:06x}]", self.title, self.get_hash()))
    }

    pub fn get<S: AsRef<str>>(
        client: &reqwest::Client,
        id: u64,
        languages: &[S],
        uid: UID,
        gid: GID,
    ) -> Result<MangaEntry, Box<dyn Error>> {
        api::MangaResponse::get(&client, id)
            .map(|response| MangaEntry {
                id: id,
                title: htmlescape::decode_html(&response.manga.title)
                    .unwrap_or(response.manga.title),
                cover: api::BASE.join(&response.manga.cover_url).unwrap(),
                chapters: response
                    .chapter
                    .into_iter()
                    .filter(|(_, chapter_field)| {
                        languages
                            .iter()
                            .any(|language| chapter_field.lang_code == language.as_ref())
                    })
                    .map(|(chapter_id, chapter_field)| ChapterInfo {
                        id: chapter_id,
                        chapter: chapter_field.chapter,
                        volume: chapter_field.volume,
                        title: htmlescape::decode_html(&chapter_field.title)
                            .unwrap_or(chapter_field.title),
                    })
                    .collect(),
                time: time::Timespec::new(chrono::offset::Utc::now().timestamp(), 0i32),
                uid: uid,
                gid: gid,
            })
            .map_err(Into::into)
    }
}

impl Entry for MangaEntry {
    fn get_entries(&self) -> Vec<fuse_mt::DirectoryEntry> {
        self.chapters
            .iter()
            .map(|chapter| fuse_mt::DirectoryEntry {
                name: std::ffi::OsString::from(chapter.format()),
                kind: fuse::FileType::Directory,
            })
            .collect()
    }

    fn read(&self, _offset: u64, _size: u32) -> Result<&[u8], libc::c_int> {
        Err(libc::ENOENT)
    }

    fn get_attributes(&self) -> fuse_mt::ResultEntry {
        Ok((
            time::Timespec::new(1, 0),
            fuse_mt::FileAttr {
                size: 4096 as u64,
                blocks: 4 as u64,
                atime: self.time,
                mtime: self.time,
                ctime: self.time,
                crtime: self.time,
                kind: fuse::FileType::Directory,
                perm: 0o444,
                nlink: self.chapters.len() as u32 + 2,
                uid: self.uid.0,
                gid: self.gid.0,
                rdev: 0 as u32,
                flags: 0,
            },
        ))
    }

    fn get_uid(&self) -> UID {
        self.uid
    }
    fn get_gid(&self) -> GID {
        self.gid
    }
}
