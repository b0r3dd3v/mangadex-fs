use crate::api;

#[derive(Debug)]
pub enum APIError {
    Request(reqwest::Error),
    NotLoggedIn
}

pub struct MangaDexAPI {
    client: reqwest::Client,
    session: Option<api::MangaDexSession>
}

pub type GetMangaError = reqwest::Error;
pub type GetChapterError = reqwest::Error;
pub type GetPageError = reqwest::Error;
pub type GetCoverError = reqwest::Error;
pub type MDListError = reqwest::Error;

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

    pub async fn get_manga(&self, id: u64) -> Result<api::Manga, GetMangaError> {
        api::Manga::get(&self.client, id).await
    }

    pub async fn get_chapter(&self, id: u64) -> Result<api::Chapter, GetChapterError> {
        api::Chapter::get(&self.client, id).await
    }

    pub async fn get_page(&self, chapter_id: u64, url: &reqwest::Url) -> Result<api::Page, GetPageError> {
        api::Page::get(&self.client, chapter_id, url).await
    }

    pub async fn get_cover(&self, url: &reqwest::Url) -> Result<api::Cover, GetPageError> {
        api::Cover::get(&self.client, url).await
    }

    pub async fn search(&self, params: &api::SearchParams) -> Result<Vec<api::SearchEntry>, api::APIError> {
        match &self.session {
            Some(session) => api::search(&self.client, &session, params).await.map_err(APIError::Request),
            None => Err(APIError::NotLoggedIn)
        }
    }

    pub async fn mdlist(&self, params: &api::MDListParams) -> Result<Vec<api::MDListEntry>, MDListError> {
        api::mdlist(&self.client, &self.session, params).await
    }

    pub async fn follow(&self, id: u64, status: &api::MDListStatus) -> Result<(), api::APIError> {
        match &self.session {
            Some(session) => api::follow(&self.client, &session, id, status).await.map_err(APIError::Request),
            None => Err(APIError::NotLoggedIn)
        }
    }

    pub async fn unfollow(&self, id: u64) -> Result<(), api::APIError> {
        match &self.session {
            Some(session) => api::unfollow(&self.client, &session, id).await.map_err(APIError::Request),
            None => Err(APIError::NotLoggedIn)
        }
    }

    pub async fn mark_chapter_read(&self, id: u64) -> Result<(), api::APIError> {
        match &self.session {
            Some(session) => api::mark_chapter_read(&self.client, &session, id).await.map_err(APIError::Request),
            None => Err(APIError::NotLoggedIn)
        }
    }

    pub async fn mark_chapter_unread(&self, id: u64) -> Result<(), api::APIError> {
        match &self.session {
            Some(session) => api::mark_chapter_unread(&self.client, &session, id).await.map_err(APIError::Request),
            None => Err(APIError::NotLoggedIn)
        }
    }

    pub async fn follows(&self) -> Result<Vec<api::FollowsEntry>, api::APIError> {
        match &self.session {
            Some(session) => api::follows(&self.client, &session).await.map_err(APIError::Request),
            None => Err(APIError::NotLoggedIn)
        }
    }
}