#[derive(Debug, Clone, Copy)]
pub struct UID(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct GID(pub u32);

pub trait Entry {
    fn get_entries(&self) -> Vec<fuse_mt::DirectoryEntry>;
    fn read(&self, offset: u64, size: u32) -> Result<&[u8], libc::c_int>;
    fn get_attributes(&self) -> fuse_mt::ResultEntry;

    fn get_uid(&self) -> UID;
    fn get_gid(&self) -> GID;
}
