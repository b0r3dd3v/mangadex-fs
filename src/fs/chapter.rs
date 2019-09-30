use chrono;
use std::error::Error;

use crate::api;
use crate::fs::chapter_info::ChapterInfo;
use crate::fs::entry::{Entry, GID, UID};

#[derive(Debug, Clone)]
pub struct Hosted {
    pub url: reqwest::Url,
    pub pages: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct External {
    pub url: reqwest::Url,
    pub file: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum Variant {
    Hosted(Hosted),
    External(External),
}

impl Hosted {
    pub fn get_page_url(&self, page: &String) -> Option<reqwest::Url> {
        if self.pages.contains(&page) {
            self.url.join(page).ok()
        } else {
            None
        }
    }
}

impl External {
    pub fn new(url: reqwest::Url) -> External {
        let file = External::generate_file(&url);

        External {
            url: url,
            file: file,
        }
    }
    fn generate_file(url: &reqwest::Url) -> Vec<u8> {
        let content = format!(
            r#"
<!DOCTYPE HTML>
<html>
  <head>
    <meta http-equiv="refresh" content="0; url={}" />
  </head>
  <body>
  </body>
</html>"#,
            url.to_string()
        );

        content.into_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct ChapterEntry {
    pub info: ChapterInfo,
    pub variant: Variant,
    pub time: time::Timespec,
    pub gid: GID,
    pub uid: UID,
}

impl ChapterEntry {
    pub fn get(
        client: &reqwest::Client,
        id: u64,
        uid: UID,
        gid: GID,
    ) -> Result<ChapterEntry, Box<dyn Error>> {
        let response = api::ChapterResponse::get(&client, id)?;

        let now = chrono::offset::Utc::now();

        return Ok(ChapterEntry {
            info: ChapterInfo {
                id,
                chapter: response.chapter,
                volume: response.volume,
                title: response.title,
            },
            time: time::Timespec::new(now.timestamp(), 0i32),
            variant: match response.external {
                Some(external) => {
                    Variant::External(External::new(reqwest::Url::parse(&external).unwrap()))
                }
                None => Variant::Hosted(Hosted {
                    url: reqwest::Url::parse(&response.server)
                        .unwrap()
                        .join(&format!("{}/", response.hash))
                        .unwrap(),
                    pages: response.page_array,
                }),
            },
            uid,
            gid,
        });
    }
}

impl Entry for ChapterEntry {
    fn get_entries(&self) -> Vec<fuse_mt::DirectoryEntry> {
        match &self.variant {
            Variant::Hosted(hosted) => hosted
                .pages
                .iter()
                .map(|page| fuse_mt::DirectoryEntry {
                    name: std::ffi::OsString::from(page),
                    kind: fuse::FileType::RegularFile,
                })
                .collect(),
            Variant::External(_) => vec![fuse_mt::DirectoryEntry {
                name: std::ffi::OsString::from("external.html"),
                kind: fuse::FileType::RegularFile,
            }],
        }
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
                nlink: match &self.variant {
                    Variant::Hosted(ref hosted) => hosted.pages.len(),
                    Variant::External(_) => 1,
                } as u32
                    + 2u32,
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
