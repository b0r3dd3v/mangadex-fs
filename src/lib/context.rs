use crate::api;
use crate::fs;

pub struct Context {
    api: tokio::sync::RwLock<api::MangaDexAPI>,
    pub manga: tokio::sync::RwLock<std::collections::HashMap<u64, std::sync::Arc<fs::entry::Manga>>>,
    pub chapters: tokio::sync::RwLock<std::collections::HashMap<u64, std::sync::Arc<fs::entry::Chapter>>>,
    pub pages: tokio::sync::RwLock<std::collections::HashMap<reqwest::Url, std::sync::Arc<fs::entry::Page>>>
}

pub enum GetOrFetch<T> {
    Cached(T),
    Fetched(T)
}

pub type GetOrFetchRef<T> = GetOrFetch<std::sync::Weak<T>>;

impl Context {
    pub fn new() -> std::sync::Arc<Context> {
        std::sync::Arc::new(Context {
            api: tokio::sync::RwLock::new(api::MangaDexAPI::new()),
            manga: tokio::sync::RwLock::new(std::collections::HashMap::default()),
            chapters: tokio::sync::RwLock::new(std::collections::HashMap::default()),
            pages: tokio::sync::RwLock::new(std::collections::HashMap::default())
        })
    }

    pub async fn log_in<L, P>(&self, login: L, password: P) -> Result<api::MangaDexSession, api::LogInError>
        where
        L: Into<std::borrow::Cow<'static, str>>,
        P: Into<std::borrow::Cow<'static, str>> {
        self.api.write().await.log_in(login, password).await.map(std::clone::Clone::clone)
    }

    pub async fn log_out(&self) -> Result<(), api::LogOutError> {
        self.api.write().await.log_out().await
    }

    pub async fn get_or_fetch_manga(&self, id: u64) -> Result<GetOrFetchRef<fs::entry::Manga>, api::AddMangaError> {
        match self.manga.write().await.entry(id) {
            std::collections::hash_map::Entry::Occupied(occupied) => Ok(GetOrFetchRef::Cached(std::sync::Arc::downgrade(occupied.get()))),
            std::collections::hash_map::Entry::Vacant(vacant) => self.api.read().await.get_manga(id).await.map(|manga| {
                GetOrFetchRef::Fetched(std::sync::Arc::downgrade(vacant.insert(std::sync::Arc::new(fs::entry::Manga::new(id, manga)))))
            })
        }
    }

    pub async fn get_or_fetch_chapter(&self, id: u64) -> Result<GetOrFetchRef<fs::entry::Chapter>, api::AddMangaError> {
        match self.chapters.write().await.entry(id) {
            std::collections::hash_map::Entry::Occupied(occupied) => Ok(GetOrFetchRef::Cached(std::sync::Arc::downgrade(occupied.get()))),
            std::collections::hash_map::Entry::Vacant(vacant) => self.api.read().await.get_chapter(id).await.map(|chapter| {
                GetOrFetchRef::Fetched(std::sync::Arc::downgrade(vacant.insert(std::sync::Arc::new(fs::entry::Chapter::new(id, chapter)))))
            })
        }
    }

    pub async fn get_or_fetch_page(&self, url: &reqwest::Url) -> Result<GetOrFetchRef<fs::entry::Page>, api::AddMangaError> {
        match self.pages.write().await.entry(url.clone()) {
            std::collections::hash_map::Entry::Occupied(occupied) => Ok(GetOrFetchRef::Cached(std::sync::Arc::downgrade(occupied.get()))),
            std::collections::hash_map::Entry::Vacant(vacant) => self.api.read().await.get_page(&url).await.map(|page| {
                GetOrFetchRef::Fetched(std::sync::Arc::downgrade(vacant.insert(std::sync::Arc::new(fs::entry::Page::ready(page)))))
            })
        }
    }

    pub async fn get_page_or_fetch_proxy(&self, url: &reqwest::Url) -> Result<GetOrFetchRef<fs::entry::Page>, api::AddMangaError> {
        match self.pages.write().await.entry(url.clone()) {
            std::collections::hash_map::Entry::Occupied(occupied) => Ok(GetOrFetchRef::Cached(std::sync::Arc::downgrade(occupied.get()))),
            std::collections::hash_map::Entry::Vacant(vacant) => self.api.read().await.get_proxy_page(&url).await.map(|page| {
                GetOrFetchRef::Fetched(std::sync::Arc::downgrade(vacant.insert(std::sync::Arc::new(fs::entry::Page::proxy(page)))))
            })
        }
    }

    pub async fn quick_search<S: AsRef<str>>(&self, query: S) -> Result<Vec<api::QuickSearchEntry>, api::QuickSearchError> {
        self.api.read().await.quick_search(query).await
    }
}