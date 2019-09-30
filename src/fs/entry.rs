pub trait Entry {
    fn get_entries(&self) -> Vec<fuse_mt::DirectoryEntry>;
    fn read(&self, offset: u64, size: u32) -> Result<&[u8], libc::c_int>;
    fn get_attributes(&self) -> fuse_mt::ResultEntry;
}