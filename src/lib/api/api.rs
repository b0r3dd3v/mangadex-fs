use crate::api;

#[derive(Debug)]
pub enum QuickSearchError {
    Request(reqwest::Error),
    NotLoggedIn
}

pub struct MangaDexAPI {
    client: reqwest::Client,
    session: Option<api::MangaDexSession>
}

pub type AddMangaError = reqwest::Error;
pub type AddChapterError = reqwest::Error;
pub type AddProxyPageError = reqwest::Error;
pub type AddPageError = reqwest::Error;

impl MangaDexAPI {
    pub fn new() -> MangaDexAPI {
        MangaDexAPI {
            client: reqwest::Client::new(),
            session: None
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

    pub async fn get_manga(&mut self, id: u64) -> Result<api::Manga, AddMangaError> {
        api::Manga::get(&self.client, id).await
    }

    pub async fn get_chapter(&mut self, id: u64) -> Result<api::Chapter, AddChapterError> {
        api::Chapter::get(&self.client, id).await
    }

    pub async fn get_proxy_page(&mut self, url: &reqwest::Url) -> Result<api::PageProxy, AddChapterError> {
        api::PageProxy::get(&self.client, url).await
    }

    pub async fn get_page(&mut self, url: &reqwest::Url) -> Result<api::Page, AddChapterError> {
        api::Page::get(&self.client, url).await
    }

    pub async fn quick_search<S: AsRef<str>>(&self, query: S) -> Result<Vec<api::QuickSearchEntry>, api::QuickSearchError> {
        match &self.session {
            Some(session) => api::quick_search(&self.client, &session, query).await.map_err(QuickSearchError::Request),
            None => Err(QuickSearchError::NotLoggedIn)
        }
    }
}