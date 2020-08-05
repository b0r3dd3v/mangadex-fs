use crate::api;
use crate::fs;

pub struct Context {
    api: std::sync::Arc<tokio::sync::Mutex<api::MangaDexAPI>>,
    entries: std::sync::Arc<tokio::sync::Mutex<fs::Entries>>,
    uid: nix::unistd::Uid,
    gid: nix::unistd::Gid
}

pub enum GetOrFetch<T> {
    Cached(T),
    Fetched(T)
}

impl Context {
    pub fn new() -> std::sync::Arc<tokio::sync::Mutex<Context>> {
        std::sync::Arc::new(tokio::sync::Mutex::new(Context {
            api: std::sync::Arc::new(tokio::sync::Mutex::new(api::MangaDexAPI::new())),
            entries: std::sync::Arc::new(tokio::sync::Mutex::new(fs::Entries::default())),
            uid: nix::unistd::Uid::current(),
            gid: nix::unistd::Gid::current()
        }))
    }

    pub async fn log_in<L, P>(&mut self, login: L, password: P) -> Result<api::MangaDexSession, api::LogInError>
        where
        L: Into<std::borrow::Cow<'static, str>>,
        P: Into<std::borrow::Cow<'static, str>> {
        self.api.lock().await.log_in(login, password).await.map(std::clone::Clone::clone)
    }

    pub async fn log_out(&mut self) -> Result<(), api::LogOutError> {
        self.api.lock().await.log_out().await
    }

    pub async fn get_or_fetch_manga(&mut self, id: u64) -> Result<GetOrFetch<String>, api::AddMangaError> {
        match self.entries.lock().await.manga.entry(id) {
            std::collections::hash_map::Entry::Occupied(occupied) => Ok(GetOrFetch::Cached(occupied.get().title.clone())),
            std::collections::hash_map::Entry::Vacant(vacant) => self.api.lock().await.get_manga(id).await.map(|manga| {
                GetOrFetch::Fetched(vacant.insert(fs::entry::Manga::new(
                    id,
                    time::Timespec::new(chrono::offset::Utc::now().timestamp(), 0i32),
                    self.uid,
                    self.gid, 
                    manga
                )).title.clone())
            })
        }
    }

    pub async fn get_or_fetch_chapter(&mut self, id: u64) -> Result<GetOrFetch<()>, api::AddMangaError> {
        match self.entries.lock().await.chapters.entry(id) {
            std::collections::hash_map::Entry::Occupied(_) => Ok(GetOrFetch::Cached(())),
            std::collections::hash_map::Entry::Vacant(vacant) => self.api.lock().await.get_chapter(id).await.map(|chapter| {
                vacant.insert(fs::entry::Chapter::new(
                    id,
                    time::Timespec::new(chrono::offset::Utc::now().timestamp(), 0i32),
                    self.uid,
                    self.gid, 
                    chapter
                ));

                GetOrFetch::Fetched(())
            })
        }
    }

    pub async fn quick_search<S: AsRef<str>>(&self, query: S) -> Result<Vec<api::QuickSearchEntry>, api::QuickSearchError> {
        self.api.lock().await.quick_search(query).await
    }
}