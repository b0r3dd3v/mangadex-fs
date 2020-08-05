pub mod chapter;
pub mod manga;
pub mod page;

pub trait Entry {
    fn read(&self, offset: u64, size: u32) -> Result<&[u8], libc::c_int>;
    fn get_entries(&self) -> Vec<fuse_mt::DirectoryEntry>;
    fn get_attributes(&self) -> fuse_mt::ResultEntry;

    fn get_uid(&self) -> u32;
    fn get_gid(&self) -> u32;
}

pub use chapter::*;
pub use manga::*;
pub use page::*;