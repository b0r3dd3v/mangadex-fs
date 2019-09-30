mod chapter;
mod chapter_info;
mod manga;
mod page;
mod entry;

pub use chapter::*;
pub use chapter_info::*;
pub use manga::*;
pub use page::*;
pub use entry::*;

use std::collections::HashMap;
use std::path::Path;

use fuse_mt;
use std::sync::{Arc, Mutex};

pub struct MangaDexFS {
    client: reqwest::Client,
    manga: HashMap<u64, MangaEntry>,
    chapters: Arc<Mutex<HashMap<u64, ChapterEntry>>>,
    pages: Arc<Mutex<HashMap<String, PageEntry>>>,
    uid: u32,
    gid: u32,
    languages: Vec<String>,
}

impl MangaDexFS {
    pub fn new() -> MangaDexFS {
        MangaDexFS {
            client: reqwest::Client::new(),
            manga: HashMap::new(),
            chapters: Arc::new(Mutex::new(HashMap::new())),
            pages: Arc::new(Mutex::new(HashMap::new())),
            uid: nix::unistd::Uid::current().as_raw() as u32,
            gid: nix::unistd::Gid::current().as_raw() as u32,
            languages: vec![],
        }
    }

    pub fn add_langauge(&mut self, lang: String) {
        info!("Adding language: {}", lang);
        self.languages.push(lang);
    }

    pub fn add_manga(&mut self, id: u64) {
        info!("Fetching manga with id {}...", id);
        match MangaEntry::get(&self.client, id, &self.languages) {
            Ok(manga) => {
                info!("Added manga: \"{}\"", &manga.title);
                self.manga.insert(id, manga);
            }
            Err(e) => {
                warn!(
                    "Failure on fetching manga with id {}, reason: {}",
                    id,
                    e.to_string()
                );
            }
        }
    }

    fn add_chapter(&self, id: u64) {
        info!("Fetching chapter with id {}...", id);
        match ChapterEntry::get(&self.client, id) {
            Ok(chapter) => {
                if let Ok(mut chapters) = self.chapters.lock() {
                    info!("Added chapter: \"{}\"", chapter.info.format());
                    chapters.insert(id, chapter);
                }
            }
            Err(e) => {
                warn!(
                    "Failure on fetching chapter with id {}, reason: {}",
                    id,
                    e.to_string()
                );
            }
        }
    }

    fn add_page(&self, chapter: &ChapterEntry, page_s: &String) {
        if let ChapterMeta::Hosted(hosted) = &chapter.meta {
            if let Some(url) = hosted.get_page_url(&page_s) {
                info!(
                    "Fetching page \"{}\" from chapter \"{}\"...",
                    &page_s,
                    chapter.info.format()
                );
                match PageEntry::get(&self.client, &url) {
                    Ok(page) => {
                        if let Ok(mut pages) = self.pages.lock() {
                            info!(
                                "Added page: \"{}\" from chapter \"{}\"",
                                &page_s,
                                chapter.info.format()
                            );
                            pages.insert(url.into_string(), page);
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failure on fetching page from \"{}\", reason: {}",
                            url.into_string(),
                            e.to_string()
                        );
                    }
                }
            } else {
                warn!(
                    "Chapter \"{}\" doesn't contain page \"{}\"",
                    chapter.info.format(),
                    page_s
                );
            }
        } else {
            debug!(
                "Attempt to fetch chapter of id \"{}\" from external host?",
                chapter.info.id
            );
        }
    }

    fn add_proxy_page(&self, chapter: &ChapterEntry, page_s: &String) {
        if let ChapterMeta::Hosted(hosted) = &chapter.meta {
            if let Some(url) = hosted.get_page_url(&page_s) {
                info!(
                    "Fetching metadata of page \"{}\" from chapter \"{}\"...",
                    &page_s,
                    chapter.info.format()
                );
                match PageEntry::get_proxy(&self.client, &url) {
                    Ok(page) => {
                        if let Ok(mut pages) = self.pages.lock() {
                            info!(
                                "Added metadata of page: \"{}\" from chapter \"{}\"",
                                &page_s,
                                chapter.info.format()
                            );
                            pages.insert(url.into_string(), page);
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failure on fetching metadata of page from \"{}\", reason: {}",
                            url.into_string(),
                            e.to_string()
                        );
                    }
                }
            } else {
                warn!(
                    "Chapter \"{}\" doesn't contain page \"{}\"",
                    chapter.info.format(),
                    page_s
                );
            }
        } else {
            debug!(
                "Attempt to fetch metadata of chapter of id \"{}\" from external host?",
                chapter.info.id
            );
        }
    }

    fn get_manga_from_path(path: &Path) -> Option<String> {
        let out = path
            .ancestors()
            .nth(path.ancestors().count() - 2)
            .take()
            .and_then(|x| x.file_name())
            .and_then(|x| x.to_str())
            .map(|x| x.into());

        out
    }

    fn get_manga(&self, path: &Path) -> Option<MangaEntry> {
        let out = MangaDexFS::get_manga_from_path(&path).and_then(|title| {
            self.manga.iter().find_map(|(_, manga)| {
                if manga.title == title {
                    Some(manga.clone())
                } else {
                    None
                }
            })
        });

        out
    }

    fn get_chapter_from_path(path: &Path) -> Option<String> {
        let out = path
            .ancestors()
            .nth(path.ancestors().count() - 3)
            .take()
            .and_then(|x| x.file_name())
            .and_then(|x| x.to_str())
            .map(|x| x.into());

        out
    }

    fn get_chapter(&self, path: &Path) -> Option<ChapterEntry> {
        let out = MangaDexFS::get_chapter_from_path(path).and_then(|chapter_name| {
            self.chapters.lock().ok().and_then(|lock| {
                lock.iter().find_map(|(_, chapter)| {
                    let formatted = chapter.info.format();
                    if formatted == chapter_name {
                        Some(chapter.clone())
                    } else {
                        None
                    }
                })
            })
        });

        out
    }

    fn get_page_name(path: &Path) -> Option<String> {
        let out = path
            .ancestors()
            .nth(path.ancestors().count() - 4)
            .take()
            .and_then(|x| x.file_name())
            .and_then(|x| x.to_str())
            .map(|x| x.into());
        out
    }

    fn get_page(&self, path: &Path) -> Option<PageEntry> {
        let chapter = self.get_chapter(&path);
        let page_name = MangaDexFS::get_page_name(&path);

        if let Some(ref page_name) = page_name {
            if let Some(ref chapter) = chapter {
                match &chapter.meta {
                    ChapterMeta::Hosted(hosted) => {
                        for page in &hosted.pages {
                            if page == page_name {
                                if let Ok(ref pages) = self.pages.lock() {
                                    return pages
                                        .get(&hosted.get_page_url(page_name).unwrap().into_string())
                                        .map(|x| x.clone());
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }

        None
    }

    fn get_or_fetch_chapter(&self, path: &Path) -> Option<ChapterEntry> {
        if let Some(chapter) = self.get_chapter(path) {
            Some(chapter)
        } else {
            if let Some(chapter_name) = MangaDexFS::get_chapter_from_path(&path) {
                if let Some(manga) = self.get_manga(&path) {
                    for chapter_info in &manga.chapters {
                        if chapter_info.format() == chapter_name {
                            self.add_chapter(chapter_info.id);
                            return self.get_chapter(path);
                        }
                    }
                }
            }

            None
        }
    }

    fn get_or_fetch_proxy_page(&self, path: &Path) -> Option<PageEntry> {
        if let Some(page) = self.get_page(&path) {
            Some(page)
        } else {
            if let Some(chapter) = self.get_or_fetch_chapter(&path) {
                if let Some(ref page_name) = MangaDexFS::get_page_name(&path) {
                    match chapter.meta {
                        ChapterMeta::Hosted(ref hosted) => {
                            for page in &hosted.pages {
                                if page == page_name {
                                    self.add_proxy_page(&chapter, &page_name);
                                    return self.get_page(&path);
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }

            None
        }
    }

    fn get_or_fetch_page(&self, path: &Path) -> Option<PageEntry> {
        if let Some(page) = self.get_page(&path) {
            match page {
                PageEntry::Proxy(_) => {
                    if let Some(chapter) = self.get_or_fetch_chapter(&path) {
                        if let Some(ref page_name) = MangaDexFS::get_page_name(&path) {
                            match chapter.meta {
                                ChapterMeta::Hosted(ref hosted) => {
                                    for page in &hosted.pages {
                                        if page == page_name {
                                            self.add_page(&chapter, &page_name);
                                            return self.get_page(&path);
                                        }
                                    }
                                }
                                _ => (),
                            }
                        }
                    }

                    None
                }
                _ => Some(page),
            }
        } else {
            if let Some(chapter) = self.get_or_fetch_chapter(&path) {
                if let Some(ref page_name) = MangaDexFS::get_page_name(&path) {
                    match chapter.meta {
                        ChapterMeta::Hosted(ref hosted) => {
                            for page in &hosted.pages {
                                if page == page_name {
                                    self.add_page(&chapter, &page_name);
                                    return self.get_page(&path);
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }

            None
        }
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

        let out = match path.ancestors().count() {
            0 => Ok(entries),
            1 => {
                for (_, manga) in &self.manga {
                    entries.push(fuse_mt::DirectoryEntry {
                        name: std::ffi::OsString::from(manga.title.clone()),
                        kind: fuse::FileType::Directory,
                    });
                }

                Ok(entries)
            }
            2 => self
                .get_manga(&path)
                .map(|manga| manga.get_entries())
                .ok_or_else(|| libc::ENOENT),
            3 => self
                .get_or_fetch_chapter(&path)
                .map(|chapter| chapter.get_entries())
                .ok_or_else(|| libc::ENOENT),
            _ => Err(libc::ENOENT),
        };

        debug!("readdir: {:?}", path);

        out
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
                if let Some(page) = self.get_or_fetch_page(&path) { return result(page.read(offset, size)); }
                else if path.file_name().unwrap().to_str().unwrap() == "external.html" {
                    if let Some(chapter) = self.get_or_fetch_chapter(&path) {
                        match chapter.meta {
                            ChapterMeta::External(external) => {
                                return result(Ok(&external.file[offset as usize
                                    ..std::cmp::min(
                                        offset as usize + size as usize,
                                        external.file.len(),
                                )]));
                            }
                            _ => return result(Err(libc::EIO))
                        }
                    }
                }
            }
            _ => ()
        };

        result(Err(libc::ENOENT));
    }

    fn getattr(
        &self,
        _req: fuse_mt::RequestInfo,
        path: &Path,
        _fh: Option<u64>,
    ) -> fuse_mt::ResultEntry {
        let out = match path.ancestors().count() {
            0 | 1 => Ok((
                time::Timespec::new(1, 0),
                fuse_mt::FileAttr {
                    size: 4096 as u64,
                    blocks: 4 as u64,
                    atime: time::Timespec::new(1, 0),
                    mtime: time::Timespec::new(1, 0),
                    ctime: time::Timespec::new(1, 0),
                    crtime: time::Timespec::new(1, 0),
                    kind: fuse::FileType::Directory,
                    perm: 0o444,
                    nlink: self.manga.len() as u32 + 2,
                    uid: self.uid,
                    gid: self.gid,
                    rdev: 0 as u32,
                    flags: 0,
                },
            )),
            2 => self
                .get_manga(&path)
                .and_then(|manga| manga.get_attributes().ok())
                .ok_or(libc::ENOENT),
            3 => self
                .get_chapter(&path)
                .and_then(|chapter| chapter.get_attributes().ok())
                .or_else(|| {
                    if let Some(manga) = self.get_manga(&path) {
                        let chapter_name = MangaDexFS::get_chapter_from_path(path).unwrap();
                        if manga
                            .chapters
                            .iter()
                            .any(|chapter| chapter_name == chapter.format())
                        {
                            Some((
                                time::Timespec::new(1, 0),
                                fuse_mt::FileAttr {
                                    size: 4096 as u64,
                                    blocks: 4 as u64,
                                    atime: manga.time,
                                    mtime: manga.time,
                                    ctime: manga.time,
                                    crtime: manga.time,
                                    kind: fuse::FileType::Directory,
                                    perm: 0o444,
                                    nlink: 2,
                                    uid: self.uid,
                                    gid: self.gid,
                                    rdev: 0 as u32,
                                    flags: 0,
                                },
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .ok_or(libc::ENOENT),
            4 => self
                .get_or_fetch_proxy_page(&path)
                .and_then(|page| page.get_attributes().ok())
                .or_else(|| {
                    if path.file_name().unwrap().to_str().unwrap() == "external.html" {
                        if let Some(chapter) = self.get_chapter(&path) {
                            match chapter.meta {
                                ChapterMeta::External(external) => {
                                    return Some((
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
                                            nlink: 1,
                                            uid: self.uid,
                                            gid: self.gid,
                                            rdev: 0 as u32,
                                            flags: 0,
                                        },
                                    ))
                                }
                                _ => ()
                            }
                        }
                    }
                    
                    None
                }).ok_or(libc::ENOENT),
            _ => Err(libc::ENOENT),
        };

        debug!("getattr: {:?}", path);

        out
    }
}
