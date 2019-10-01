mod chapter;
mod chapter_info;
mod entry;
mod manga;
mod page;

pub use chapter::{ChapterEntry, External, Hosted, Variant as ChapterVariant};
pub use chapter_info::*;
pub use entry::*;
pub use manga::*;
pub use page::{PageEntry, Variant as PageVariant};

use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

use fuse_mt;
use std::sync::{Arc, Mutex};

pub struct MangaDexFS {
    client: reqwest::Client,
    manga: HashMap<u64, MangaEntry>,
    chapters: Arc<Mutex<HashMap<u64, ChapterEntry>>>,
    pages: Arc<Mutex<HashMap<String, PageEntry>>>,
    uid: UID,
    gid: GID,
    time: time::Timespec,
    languages: Vec<String>,
}

impl MangaDexFS {
    pub fn new() -> MangaDexFS {
        MangaDexFS {
            client: reqwest::Client::new(),
            manga: HashMap::new(),
            chapters: Arc::new(Mutex::new(HashMap::new())),
            pages: Arc::new(Mutex::new(HashMap::new())),
            uid: UID(nix::unistd::Uid::current().as_raw() as u32),
            gid: GID(nix::unistd::Gid::current().as_raw() as u32),
            languages: vec![],
            time: time::Timespec::new(chrono::offset::Utc::now().timestamp(), 0i32)
        }
    }

    pub fn add_language<S: Into<String>>(&mut self, lang: S) {
        let lang = lang.into();

        info!("Adding language: {}", &lang);
        self.languages.push(lang);
    }
    
    pub fn add_manga(&mut self, id: u64) -> Result<(), Box<dyn Error>> {
        MangaEntry::get(&self.client, id, &self.languages, self.uid, self.gid)
            .map(|manga| {
                self.manga.insert(id, manga);
                ()
            })
            .map_err(Into::into)
    }

    fn add_chapter(&self, id: u64) -> Result<(), Box<dyn Error>> {
        ChapterEntry::get(&self.client, id, self.uid, self.gid)
            .map_err(Into::into)
            .and_then(|chapter| {
                self.chapters
                    .lock()
                    .as_mut()
                    .map(|chapters| {
                        chapters.insert(id, chapter);
                        ()
                    })
                    .map_err(|e| e.to_string().into())
            })
    }

    fn add_page<S: AsRef<str>>(&self, hosted: &Hosted, page_s: S) -> Result<(), Box<dyn Error>> {
        hosted.get_page_url(page_s.as_ref()).and_then(|url| {
            PageEntry::get(&self.client, &url, self.uid, self.gid).and_then(|page| {
                self.pages
                    .lock()
                    .as_mut()
                    .map(|pages| {
                        pages.insert(url.into_string(), page);
                        ()
                    })
                    .map_err(|e| e.to_string().into())
            })
        })
    }

    fn add_proxy_page<S: Into<String>>(
        &self,
        hosted: &Hosted,
        page_s: S,
    ) -> Result<(), Box<dyn Error>> {
        hosted.get_page_url(page_s.into()).and_then(|url| {
            PageEntry::get_proxy(&self.client, &url, self.uid, self.gid).and_then(|page| {
                self.pages
                    .lock()
                    .as_mut()
                    .map(|pages| {
                        pages.insert(url.into_string(), page);
                        ()
                    })
                    .map_err(|e| e.to_string().into())
            })
        })
    }

    #[allow(dead_code)]
    fn get_manga_by_id(&self, id: u64) -> Option<MangaEntry> {
        self.manga.get(&id).map(Clone::clone)
    }

    fn get_manga<P: AsRef<Path>>(&self, path: P) -> Option<MangaEntry> {
        MangaDexFS::get_nth_child(&path, 2).and_then(|title| {
            self.manga.values().find(|manga| manga.format() == title).map(Clone::clone)
        })
    }

    fn get_chapter_by_id(&self, id: u64) -> Result<ChapterEntry, Box<dyn Error>> {
        self.chapters
            .lock()
            .map_err(|e| e.to_string().into())
            .and_then(|lock| {
                lock.get(&id)
                    .map(Clone::clone)
                    .ok_or(format!("Chapter of id {} not found", id).into())
            })
    }

    fn get_chapter<P: AsRef<Path>>(&self, path: P) -> Option<ChapterEntry> {
        MangaDexFS::get_nth_child(&path, 3).and_then(|chapter_name| {
            self.chapters.lock().ok().and_then(|lock| {
                lock.values()
                    .find(|chapter| chapter.info.format() == chapter_name)
                    .map(Clone::clone)
            })
        })
    }

    fn get_nth_child<'a, P: AsRef<Path>>(path: &'a P, n: usize) -> Option<&'a str> {
        path.as_ref()
            .ancestors()
            .nth(path.as_ref().ancestors().count() - n)
            .take()
            .and_then(|x| x.file_name())
            .and_then(|x| x.to_str())
    }

    #[allow(dead_code)]
    fn get_page_by_url(&self, url: &reqwest::Url) -> Result<PageEntry, Box<dyn Error>> {
        self.pages
            .lock()
            .map_err(|e| e.to_string().into())
            .and_then(|lock| {
                lock.get(&url.to_string())
                    .map(Clone::clone)
                    .ok_or("Page not found".into())
            })
    }

    fn get_page<P: AsRef<Path>>(
        &self,
        hosted: &Hosted,
        path: P,
    ) -> Result<PageEntry, Box<dyn Error>> {
        MangaDexFS::get_nth_child(&path, 4)
            .ok_or("Invalid path.".into())
            .and_then(|page| {
                self.pages
                    .lock()
                    .map_err(|e| e.to_string().into())
                    .and_then(|pages| {
                        hosted.get_page_url(page.to_string()).and_then(|url| {
                            pages
                                .get(&url.to_string())
                                .map(Clone::clone)
                                .ok_or("Page not found".into())
                        })
                    })
            })
    }

    fn get_or_fetch_chapter<'a, P: AsRef<Path>>(
        &self,
        path: &'a P,
    ) -> Result<ChapterEntry, Box<dyn Error>> {
        self.get_chapter(&path)
            .ok_or("Chapter hasn't been fetched yet.".into())
            .or_else(|_: Box<dyn Error>| {
                MangaDexFS::get_nth_child(&path, 3)
                    .ok_or("Invalid path.".into())
                    .and_then(|chapter_name| {
                        self.get_manga(&path)
                            .ok_or("Manga not found.".into())
                            .and_then(|manga| {
                                manga
                                    .chapters
                                    .iter()
                                    .find(|chapter_info| chapter_info.format() == chapter_name)
                                    .ok_or("Chapter not found.".into())
                                    .and_then(|chapter_info| {
                                        self.add_chapter(chapter_info.id)
                                            .and_then(|_| self.get_chapter_by_id(chapter_info.id))
                                    })
                            })
                    })
            })
    }

    fn get_or_fetch_proxy_page<'a, P: AsRef<Path>>(
        &self,
        hosted: &Hosted,
        path: &'a P,
    ) -> Result<PageEntry, Box<dyn Error>> {
        self.get_page(hosted, path).or_else(|_| {
            MangaDexFS::get_nth_child(&path, 4)
                .ok_or("Invalid path.".into())
                .and_then(|page_name| self.add_proxy_page(hosted, page_name))
                .and_then(|_| self.get_page(hosted, path.as_ref()))
        })
    }

    fn get_or_fetch_page<P: AsRef<Path>>(
        &self,
        hosted: &Hosted,
        path: P,
    ) -> Result<PageEntry, Box<dyn Error>> {
        self.get_page(hosted, &path)
            .and_then(|entry| match entry.variant {
                PageVariant::Ready { data: _ } => Ok(entry),
                PageVariant::Proxy { size: _ } => MangaDexFS::get_nth_child(&path, 4)
                    .ok_or("Invalid path.".into())
                    .and_then(|page_name| {
                        hosted
                            .get_page_url(page_name)
                            .and_then(|_| self.add_page(hosted, page_name))
                            .and_then(|_| self.get_page(hosted, &path))
                    }),
            })
    }
}

impl fuse_mt::FilesystemMT for MangaDexFS {
    fn init(&self, _req: fuse_mt::RequestInfo) -> fuse_mt::ResultEmpty {
        Ok(())
    }

    fn destroy(&self, _req: fuse_mt::RequestInfo) {}

    fn opendir(&self, _req: fuse_mt::RequestInfo, path: &Path, flags: u32) -> fuse_mt::ResultOpen {
        debug!("opendir: {:?} (flags = {:#o})", path, flags);

        Ok((0, flags))
    }

    fn readdir(&self, _req: fuse_mt::RequestInfo, path: &Path, _fh: u64) -> fuse_mt::ResultReaddir {
        let mut entries: Vec<fuse_mt::DirectoryEntry> = vec![];

        entries.push(fuse_mt::DirectoryEntry {
            name: std::ffi::OsString::from("."),
            kind: fuse::FileType::Directory,
        });
        entries.push(fuse_mt::DirectoryEntry {
            name: std::ffi::OsString::from(".."),
            kind: fuse::FileType::Directory,
        });

        let level = path.ancestors().count();

        if level >= 4 {
            warn!("Path {:?} is too deep.", path);
            return Err(libc::ENOENT);
        }

        let entries_found = match level {
            1 => self.manga.iter().map(|(_, manga)| {
                Ok(fuse_mt::DirectoryEntry {
                    name: std::ffi::OsString::from(manga.format()),
                    kind: fuse::FileType::Directory,
                })
            }).collect(),
            2 => self
                .get_manga(&path)
                .map(|manga| manga.get_entries())
                .ok_or("Manga not found.".into()),
            3 => self
                .get_or_fetch_chapter(&path)
                .map(|chapter| chapter.get_entries()),
            _ => Ok(vec![])
        };
        
        match entries_found {
            Ok(found) => {
                for entry in found {
                    entries.push(entry);
                }
            },
            Err(e) => {
                warn!("readdir of path \"{:?}\": {}", path, e);
            }
        };

        debug!("readdir: {:?}", path);

        Ok(entries)
    }

    fn read(
        &self,
        _req: fuse_mt::RequestInfo,
        path: &Path,
        _fh: u64,
        offset: u64,
        size: u32,
        result: impl FnOnce(Result<&[u8], libc::c_int>),
    ) {
        debug!("read: {:?} {:#x} @ {:#x}", path, size, offset);

        match path.ancestors().count() {
            4 => {
                match self.get_or_fetch_chapter(&path) {
                    Ok(chapter) => match chapter.variant {
                        ChapterVariant::Hosted(hosted) => {
                            match self.get_or_fetch_page(&hosted, path) {
                                Ok(page) => result(page.read(offset, size)),
                                _ => result(Err(libc::ENOENT)),
                            }
                        }
                        ChapterVariant::External(external) => {
                            match path.file_name().and_then(std::ffi::OsStr::to_str) {
                                Some(filename) => {
                                    if filename == "external.html" {
                                        result(Ok(&external.file[offset as usize
                                            ..std::cmp::min(
                                                offset as usize + size as usize,
                                                external.file.len(),
                                            )]));
                                    } else {
                                        result(Err(libc::ENOENT)); // TODO: These result(Err(...) branches look non-idiomiatic, but result is FnOnce?
                                    }
                                }
                                _ => result(Err(libc::ENOENT)),
                            }
                        }
                    },
                    _ => result(Err(libc::ENOENT)),
                }
            }
            _ => result(Err(libc::ENOENT)),
        };
    }

    fn getattr(
        &self,
        _req: fuse_mt::RequestInfo,
        path: &Path,
        _fh: Option<u64>,
    ) -> fuse_mt::ResultEntry {
        debug!("getattr: {:?}", path);

        match path.ancestors().count() {
            0 | 1 => Ok((
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
                    nlink: self.manga.len() as u32 + 2,
                    uid: self.uid.0,
                    gid: self.gid.0,
                    rdev: 0 as u32,
                    flags: 0,
                },
            )),
            2 => self
                .get_manga(&path)
                .ok_or(libc::ENOENT)
                .and_then(|manga| manga.get_attributes()),
            3 => self
                .get_chapter(&path)
                .ok_or(libc::ENOENT)
                .and_then(|chapter| chapter.get_attributes())
                .or_else(|_| {
                    MangaDexFS::get_nth_child(&path, 3)
                        .ok_or(libc::ENOENT)
                        .and_then(|chapter_name| {
                            self.get_manga(&path).ok_or(libc::ENOENT).and_then(|manga| {
                                manga
                                    .chapters
                                    .iter()
                                    .find(|chapter_info| chapter_info.format() == chapter_name)
                                    .ok_or(libc::ENOENT)
                                    .and_then(|_| manga.get_attributes())
                            })
                        })
                }),
            4 => self
                .get_chapter(&path)
                .ok_or(libc::ENOENT)
                .and_then(|chapter| match chapter.variant {
                    ChapterVariant::Hosted(ref hosted) => self
                        .get_or_fetch_proxy_page(hosted, &path)
                        .map_err(|_| libc::ENOENT)
                        .and_then(|page| page.get_attributes()),
                    ChapterVariant::External(ref external) => Ok((
                        time::Timespec::new(1, 0),
                        fuse_mt::FileAttr {
                            size: external.file.len() as u64,
                            blocks: (1f64 + external.file.len() as f64 / 4f64) as u64,
                            atime: chapter.time,
                            mtime: chapter.time,
                            ctime: chapter.time,
                            crtime: chapter.time,
                            kind: fuse::FileType::RegularFile,
                            perm: 0o444,
                            nlink: 1u32,
                            uid: chapter.uid.0,
                            gid: chapter.gid.0,
                            rdev: 0 as u32,
                            flags: 0,
                        },
                    )),
                }),
            _ => Err(libc::ENOENT),
        }
    }
}
