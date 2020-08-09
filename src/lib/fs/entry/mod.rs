pub mod chapter;
pub mod manga;
pub mod page;

pub use chapter::*;
pub use manga::*;
pub use page::*;

#[derive(Debug)]
pub struct Attributes {
    ino: u64,
    atime: std::time::SystemTime,
    mtime: std::time::SystemTime,
    ctime: std::time::SystemTime,
    uid: nix::unistd::Uid,
    gid: nix::unistd::Gid
}


impl Attributes {
    pub fn new(ino: u64, uid: nix::unistd::Uid, gid: nix::unistd::Gid) -> Attributes {
        Attributes {
            ino,
            atime: std::time::SystemTime::now(),
            mtime: std::time::SystemTime::now(),
            ctime: std::time::SystemTime::now(),
            uid,
            gid
        }
    }
}

#[derive(Debug)]
pub struct Directory {
    pub children: std::collections::HashMap<std::path::PathBuf, (u64, bool)>,
    pub parent: Option<u64>,
}

impl Directory {
    pub fn new(parent: u64) -> Directory {
        Directory {
            children: std::collections::HashMap::default(),
            parent: Some(parent)
        }
    }

    pub fn root() -> Directory {
        Directory {
            children: std::collections::HashMap::default(),
            parent: None
        }
    }

    pub fn entries(&self) -> Vec<polyfuse::DirEntry> {
        self.children
            .iter()
            .enumerate()
            .map(|(index, (path, (ino, is_file)))| {
                if *is_file {
                    polyfuse::DirEntry::file(path, *ino, index as u64 + 3u64)
                }
                else {
                    polyfuse::DirEntry::dir(path, *ino, index as u64 + 3u64)
                }
            })
            .collect::<Vec<_>>()
    }
}

impl Attributes {
    pub fn file_attr(&self) -> polyfuse::FileAttr {
        let mut attr = polyfuse::FileAttr::default();

        attr.set_ino(self.ino);
        attr.set_atime(self.atime);
        attr.set_mtime(self.mtime);
        attr.set_ctime(self.ctime);
        attr.set_uid(self.uid.as_raw() as u32);
        attr.set_gid(self.gid.as_raw() as u32);
        attr.set_rdev(0u32);

        attr
    }
}

#[derive(Debug)]
pub enum Entry {
    Manga(std::sync::Weak<Manga>, Directory),
    Chapter(std::sync::Weak<Chapter>, Directory),
    ChapterNotFetched(u64),
    Page(std::sync::Weak<Page>),
    External(Vec<u8>),
    Root(Directory)
}

#[derive(Debug)]
pub struct Inode(pub Entry, pub Attributes);

impl Inode {
    pub fn get_attr(&self) -> Option<polyfuse::FileAttr> {
        let (entry, attributes) = (&self.0, &self.1);

        match entry {
            Entry::Manga(manga_ref, _) => manga_ref.upgrade().map(|manga| {
                let mut attr = attributes.file_attr();

                attr.set_size(4096u64);
                attr.set_blocks(8u64);
                attr.set_mode(libc::S_IFDIR as u32 | 0o555);
                attr.set_nlink(2u32 + manga.chapters.len() as u32);

                attr
            }),
            Entry::Chapter(chapter_ref, _) => chapter_ref.upgrade().map(|chapter| {
                let mut attr = attributes.file_attr();

                attr.set_size(4096u64);
                attr.set_blocks(8u64);
                attr.set_mode(libc::S_IFDIR as u32 | 0o555);
                match &chapter.pages {
                    ChapterPages::Hosted(hosted) => attr.set_nlink(2u32 + hosted.pages.len() as u32),
                    ChapterPages::External(_) => attr.set_nlink(2u32 + 1u32),
                };

                attr
            }),
            Entry::ChapterNotFetched(_) => {
                let mut attr = attributes.file_attr();

                attr.set_size(4096u64);
                attr.set_blocks(8u64);
                attr.set_mode(libc::S_IFDIR as u32 | 0o555);
                attr.set_nlink(2u32);

                Some(attr)
            },
            Entry::Page(page_ref) => page_ref.upgrade().map(|page| {
                let mut attr = attributes.file_attr();

                attr.set_size(page.0.len() as u64);
                attr.set_blocks(4u64);
                attr.set_mode(libc::S_IFREG as u32 | 0o444);
                attr.set_nlink(1u32);

                attr
            }),
            Entry::External(bytes) => Some({
                let mut attr = attributes.file_attr();

                attr.set_size(bytes.len() as u64);
                attr.set_blocks(4u64);
                attr.set_mode(libc::S_IFREG as u32 | 0o444);
                attr.set_nlink(1u32);

                attr
            }),
            Entry::Root(directory) => {
                let mut attr = attributes.file_attr();

                attr.set_size(4096u64);
                attr.set_blocks(8u64);
                attr.set_mode(libc::S_IFDIR as u32 | 0o555);
                attr.set_nlink(2u32 + directory.children.len() as u32);
                
                Some(attr)
            }
        }
    }
}