use crate::fs::entry;

#[derive(Default)]
pub struct Entries {
    pub manga: std::collections::HashMap<u64, entry::Manga>,
    pub chapters: std::collections::HashMap<u64, entry::Chapter>,
    pub pages: std::collections::HashMap<reqwest::Url, entry::Page>
}

impl Entries {
    pub fn new() -> Entries {
        Entries::default()
    }
}