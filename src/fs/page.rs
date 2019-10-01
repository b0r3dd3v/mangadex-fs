use chrono;
use std::error::Error;

use crate::fs::entry::{Entry, GID, UID};

#[derive(Debug, Clone)]
pub enum Variant {
    Proxy { size: u64 },
    Ready { data: Vec<u8> },
}

impl Variant {
    fn get_size(&self) -> u64 {
        match &self {
            Variant::Proxy { ref size } => *size,
            Variant::Ready { ref data } => data.len() as u64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PageEntry {
    pub variant: Variant,
    pub time: time::Timespec,
    pub uid: UID,
    pub gid: GID,
}

impl PageEntry {
    pub fn get_size(&self) -> u64 {
        self.variant.get_size()
    }
}

impl Entry for PageEntry {
    fn get_entries(&self) -> Vec<fuse_mt::DirectoryEntry> {
        vec![]
    }

    fn read(&self, offset: u64, size: u32) -> Result<&[u8], libc::c_int> {
        match &self.variant {
            Variant::Ready { ref data } => {
                Ok(&data
                    [offset as usize..std::cmp::min(offset as usize + size as usize, data.len())])
            }
            Variant::Proxy { size: _ } => Err(libc::EIO),
        }
    }

    fn get_attributes(&self) -> fuse_mt::ResultEntry {
        Ok((
            time::Timespec::new(1, 0),
            fuse_mt::FileAttr {
                size: self.get_size() as u64,
                blocks: 4 as u64,
                atime: self.time,
                mtime: self.time,
                ctime: self.time,
                crtime: self.time,
                kind: fuse::FileType::RegularFile,
                perm: 0o444,
                nlink: 1u32,
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

impl PageEntry {
    pub fn get_proxy(
        client: &reqwest::Client,
        url: &reqwest::Url,
        uid: UID,
        gid: GID,
    ) -> Result<PageEntry, Box<dyn Error>> {
        let response = client.head(url.as_ref()).send()?;
        let headers = response.headers();
        let content_length = &headers[reqwest::header::CONTENT_LENGTH];

        let size = content_length.to_str().unwrap().parse::<u64>().unwrap(); 

        return Ok(PageEntry {
            variant: Variant::Proxy { size },
            uid,
            gid,
            time: time::Timespec::new(chrono::offset::Utc::now().timestamp(), 0i32),
        });
    }

    pub fn get(
        client: &reqwest::Client,
        url: &reqwest::Url,
        uid: UID,
        gid: GID,
    ) -> Result<PageEntry, Box<dyn Error>> {
        let mut response = client.get(url.as_ref()).send()?;
        let mut data: Vec<u8> = vec![];

        std::io::copy(&mut response, &mut data).unwrap();

        return Ok(PageEntry {
            variant: Variant::Ready { data },
            uid,
            gid,
            time: time::Timespec::new(chrono::offset::Utc::now().timestamp(), 0i32),
        });
    }
}
