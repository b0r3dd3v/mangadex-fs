use chrono;
use std::error::Error;

use crate::fs::entry::Entry;

#[derive(Debug, Clone)]
pub struct Proxy {
    pub size: u64,
    pub time: time::Timespec,
}

#[derive(Debug, Clone)]
pub struct Page {
    pub data: Vec<u8>,
    pub time: time::Timespec,
}

#[derive(Debug, Clone)]
pub enum PageEntry {
    Proxy(Proxy),
    Ready(Page),
}

impl PageEntry {
    pub fn get_size(&self) -> u64 {
        match self {
            PageEntry::Proxy(proxy) => proxy.size,
            PageEntry::Ready(ready) => ready.data.len() as u64,
        }
    }

    pub fn get_time(&self) -> time::Timespec {
        match self {
            PageEntry::Proxy(proxy) => proxy.time,
            PageEntry::Ready(ready) => ready.time,
        }
    }
}

impl Entry for PageEntry {
    fn get_entries(&self) -> Vec<fuse_mt::DirectoryEntry> {
        vec![]
    }

    fn read(&self, offset: u64, size: u32) -> Result<&[u8], libc::c_int> {
        match self {
            PageEntry::Ready(ready) => Ok(&ready.data[offset as usize
                ..std::cmp::min(offset as usize + size as usize, ready.data.len())]),
            PageEntry::Proxy(_) => Err(libc::EIO),
        }
    }

    fn get_attributes(&self) -> fuse_mt::ResultEntry {
        Ok((
            time::Timespec::new(1, 0),
            fuse_mt::FileAttr {
                size: self.get_size() as u64,
                blocks: 4 as u64,
                atime: self.get_time(),
                mtime: self.get_time(),
                ctime: self.get_time(),
                crtime: self.get_time(),
                kind: fuse::FileType::RegularFile,
                perm: 0o444,
                nlink: 1u32,
                uid: 0u32,
                gid: 0u32,
                rdev: 0 as u32,
                flags: 0,
            },
        ))
    }
}

impl PageEntry {
    pub fn get_proxy(
        client: &reqwest::Client,
        url: &reqwest::Url,
    ) -> Result<PageEntry, Box<dyn Error>> {
        let response = client.head(url.as_ref()).send()?;

        let now = chrono::offset::Utc::now();

        let headers = response.headers();
        let content_length = &headers[reqwest::header::CONTENT_LENGTH];

        let size = content_length.to_str().unwrap().parse::<u64>().unwrap();

        return Ok(PageEntry::Proxy(Proxy {
            size,
            time: time::Timespec::new(now.timestamp(), 0i32),
        }));
    }

    pub fn get(client: &reqwest::Client, url: &reqwest::Url) -> Result<PageEntry, Box<dyn Error>> {
        let mut response = client.get(url.as_ref()).send()?;

        let now = chrono::offset::Utc::now();

        let mut data: Vec<u8> = vec![];

        std::io::copy(&mut response, &mut data).unwrap();

        return Ok(PageEntry::Ready(Page {
            data,
            time: time::Timespec::new(now.timestamp(), 0i32),
        }));
    }
}
