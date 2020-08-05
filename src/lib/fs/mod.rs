pub mod entry;
pub mod entries;

pub use entries::*;

use crate::Context;

pub struct MangaDexFS {
    context: std::sync::Arc<tokio::sync::Mutex<Context>>,
    uid: nix::unistd::Uid,
    gid: nix::unistd::Gid,
    time: time::Timespec
}

impl MangaDexFS {
    pub fn new(
        context: std::sync::Arc<tokio::sync::Mutex<Context>>
    ) -> MangaDexFS {
        MangaDexFS {
            context,
            uid: nix::unistd::Uid::current(),
            gid: nix::unistd::Gid::current(),
            time: time::Timespec::new(chrono::offset::Utc::now().timestamp(), 0i32)
        }
    }
}

impl fuse_mt::FilesystemMT for MangaDexFS {
    fn init(&self, _req: fuse_mt::RequestInfo) -> fuse_mt::ResultEmpty {
        Ok(())
    }

    fn destroy(&self, _req: fuse_mt::RequestInfo) {}

    fn opendir(&self, _req: fuse_mt::RequestInfo, path: &std::path::Path, flags: u32) -> fuse_mt::ResultOpen {
        debug!("opendir: {:?} (flags = {:#o})", path, flags);

        Ok((0, flags))
    }

    fn readdir(&self, _req: fuse_mt::RequestInfo, path: &std::path::Path, _fh: u64) -> fuse_mt::ResultReaddir {
        let mut entries: Vec<fuse_mt::DirectoryEntry> = vec![];

        entries.push(fuse_mt::DirectoryEntry {
            name: std::ffi::OsString::from("."),
            kind: fuse::FileType::Directory,
        });
        entries.push(fuse_mt::DirectoryEntry {
            name: std::ffi::OsString::from(".."),
            kind: fuse::FileType::Directory,
        });

        debug!("readdir: {:?}", path);

        Ok(entries)
    }

    fn read(
        &self,
        _req: fuse_mt::RequestInfo,
        path: &std::path::Path,
        _fh: u64,
        offset: u64,
        size: u32,
        result: impl FnOnce(Result<&[u8], libc::c_int>),
    ) {
        debug!("read: {:?} {:#x} @ {:#x}", path, size, offset);

        result(Err(libc::ENOENT));
    }

    fn getattr(
        &self,
        _req: fuse_mt::RequestInfo,
        path: &std::path::Path,
        _fh: Option<u64>,
    ) -> fuse_mt::ResultEntry {
        debug!("getattr: {:?}", path);

        match path.to_str() {
            Some("/") => Ok((
                time::Timespec::new(1, 0),
                fuse_mt::FileAttr {
                    size: 4096u64,
                    blocks: 4u64,
                    atime: self.time,
                    mtime: self.time,
                    ctime: self.time,
                    crtime: self.time,
                    kind: fuse::FileType::Directory,
                    perm: 0o444,
                    nlink: 2,
                    uid: self.uid.as_raw() as u32,
                    gid: self.gid.as_raw() as u32,
                    rdev: 0u32,
                    flags: 0,
                },
            )),
            _ => Err(libc::ENOENT)
        }
    }
}