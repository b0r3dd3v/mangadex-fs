use crate::api;

#[derive(Default)]
struct CachedRequests {
    manga: std::collections::HashMap<u64, api::Manga>,
    chapters: std::collections::HashMap<u64, api::Chapter>
}

#[derive(Debug)]
pub enum QuickSearchError {
    Request(reqwest::Error),
    NotLoggedIn
}

pub struct MangaDexAPI {
    client: reqwest::Client,
    session: Option<api::MangaDexSession>,
    cached: CachedRequests
}

pub type AddMangaError = reqwest::Error;
pub type AddChapterError = reqwest::Error;

impl MangaDexAPI {
    pub fn new() -> MangaDexAPI {
        MangaDexAPI {
            client: reqwest::Client::new(),
            session: None,
            cached: CachedRequests::default()
        }
    }

    pub async fn log_out(&mut self) -> Result<(), api::LogOutError> {
        if let Some(session) = &self.session {
            let result = api::MangaDexSession::log_out(&self.client, session).await;

            if result.is_ok() {
                self.session = None;
            }

            result
        }
        else {
            Ok(())
        }
    }

    pub async fn log_in<L, P>(&mut self, login: L, password: P) -> Result<&api::MangaDexSession, api::LogInError>
        where
            L: Into<std::borrow::Cow<'static, str>>,
            P: Into<std::borrow::Cow<'static, str>> {
        let result = api::MangaDexSession::log_in(&self.client, login, password).await;

        match result {
            Ok(session) => {
                self.session = Some(session);
                Ok(&self.session.as_ref().unwrap())
            }
            Err(error) => Err(error)
        }
    }

    pub async fn get_manga(&mut self, id: u64) -> Result<&api::Manga, AddMangaError> {
        if !self.cached.manga.contains_key(&id) {
            match api::Manga::get(&self.client, id).await {
                Ok(manga) => { self.cached.manga.insert(id, manga); },
                Err(error) => return Err(error)
            }
        }

        Ok(self.cached.manga.get(&id).unwrap())
    }

    pub async fn get_chapter(&mut self, id: u64) -> Result<&api::Chapter, AddChapterError> {
        if !self.cached.chapters.contains_key(&id) {
            match api::Chapter::get(&self.client, id).await {
                Ok(chapter) => { self.cached.chapters.insert(id, chapter); },
                Err(error) => return Err(error)
            }
        }

        Ok(self.cached.chapters.get(&id).unwrap())
    }

    pub async fn quick_search<S: AsRef<str>>(&self, query: S) -> Result<Vec<api::QuickSearchEntry>, api::QuickSearchError> {
        match &self.session {
            Some(session) => api::quick_search(&self.client, &session, query).await.map_err(QuickSearchError::Request),
            None => Err(QuickSearchError::NotLoggedIn)
        }
    }
}