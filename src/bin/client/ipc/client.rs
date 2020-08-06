use mangadex_fs::ipc;
use mangadex_fs::api;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;

pub struct Client {
    stream: tokio::net::UnixStream
}

pub enum ClientError {
    IO(std::io::Error),
    Message(String)
}

pub type ClientResult<R> = Result<R, ClientError>;

impl Client {
    pub fn new(stream: tokio::net::UnixStream) -> Client {
        Client {
            stream: stream
        }
    }

    pub async fn read_response(&mut self) -> ClientResult<ipc::Response> {
        self.stream.read_u8().await.map_err(ClientError::IO)
    }

    pub async fn kill(&mut self) -> ClientResult<()> {
        self.stream.write_u8(ipc::KILL).await.map_err(ClientError::IO)
    }

    pub async fn log_in<L: AsRef<str>, P: AsRef<str>>(&mut self, username: L, password: P) -> ClientResult<()> {
        self.stream.write_u8(ipc::LOG_IN).await.map_err(ClientError::IO)?;
        ipc::write_string(&mut self.stream, username).await.map_err(ClientError::IO)?;
        ipc::write_string(&mut self.stream, password).await.map_err(ClientError::IO)?;
        self.stream.flush().await.map_err(ClientError::IO)?;

        match self.read_response().await? {
            ipc::LOG_IN_RESULT => match self.read_response().await? {
                ipc::LOG_IN_RESULT_OK => Ok(()),
                ipc::LOG_IN_RESULT_ERROR_REQUEST => Err(ClientError::Message("request error".into())),
                ipc::LOG_IN_RESULT_ERROR_RESPONSE => {
                    let body = ipc::read_string(&mut self.stream).await.map_err(ClientError::IO)?;

                    Err(ClientError::Message(format!("MangaDex response: {}", body)))
                },
                ipc::LOG_IN_RESULT_ERROR_INVALID => Err(ClientError::Message("invalid MangaDex response".into())),
                _ => Err(ClientError::Message("invalid daemon response".into()))
            },
            _ => Err(ClientError::Message("invalid daemon response".into()))
        }
    }

    pub async fn log_out(&mut self) -> ClientResult<()> {
        self.stream.write_u8(ipc::LOG_OUT).await.map_err(ClientError::IO)?;
        self.stream.flush().await.map_err(ClientError::IO)?;

        match self.read_response().await? {
            ipc::LOG_OUT_RESULT => match self.read_response().await? {
                ipc::LOG_OUT_RESULT_OK => Ok(()),
                ipc::LOG_OUT_RESULT_ERROR_REQUEST => Err(ClientError::Message("request error".into())),
                ipc::LOG_OUT_RESULT_ERROR_RESPONSE => {
                    let body = ipc::read_string(&mut self.stream).await.map_err(ClientError::IO)?;

                    Err(ClientError::Message(format!("MangaDex response: {}", body)))
                },
                _ => Err(ClientError::Message("invalid daemon response".into()))
            },
            _ => Err(ClientError::Message("invalid daemon response".into()))
        }
    }

    pub async fn add_manga(&mut self, manga_id: u64) -> ClientResult<mangadex_fs::GetOrFetch<String>> {
        self.stream.write_u8(ipc::ADD_MANGA).await.map_err(ClientError::IO)?;
        self.stream.write_u64(manga_id).await.map_err(ClientError::IO)?;
        self.stream.flush().await.map_err(ClientError::IO)?;

        match self.read_response().await? {
            ipc::ADD_MANGA_RESULT => match self.read_response().await? {
                ipc::ADD_MANGA_RESULT_OK_CACHE => {
                    let title = ipc::read_string(&mut self.stream).await.map_err(ClientError::IO)?;

                    Ok(mangadex_fs::GetOrFetch::Cached(title))
                },
                ipc::ADD_MANGA_RESULT_OK_FETCH => {
                    let title = ipc::read_string(&mut self.stream).await.map_err(ClientError::IO)?;

                    Ok(mangadex_fs::GetOrFetch::Fetched(title))
                },
                ipc::ADD_MANGA_RESULT_ERROR_DROPPED => Err(ClientError::Message("pointer dropped".into())),
                ipc::ADD_MANGA_RESULT_ERROR_REQUEST => Err(ClientError::Message("request error".into())),
                _ => Err(ClientError::Message("invalid daemon response".into()))
            },
            _ => Err(ClientError::Message("invalid daemon response".into()))
        }
    }

    pub async fn add_chapter(&mut self, chapter_id: u64) -> ClientResult<mangadex_fs::GetOrFetch<()>> {
        self.stream.write_u8(ipc::ADD_CHAPTER).await.map_err(ClientError::IO)?;
        self.stream.write_u64(chapter_id).await.map_err(ClientError::IO)?;
        self.stream.flush().await.map_err(ClientError::IO)?;
        
        match self.read_response().await? {
            ipc::ADD_CHAPTER_RESULT => match self.read_response().await? {
                ipc::ADD_CHAPTER_RESULT_OK_CACHE => Ok(mangadex_fs::GetOrFetch::Cached(())),
                ipc::ADD_CHAPTER_RESULT_OK_FETCH => Ok(mangadex_fs::GetOrFetch::Fetched(())),
                ipc::ADD_CHAPTER_RESULT_ERROR_DROPPED => Err(ClientError::Message("pointer dropped".into())),
                ipc::ADD_CHAPTER_RESULT_ERROR_REQUEST => Err(ClientError::Message("request error".into())),
                _ => Err(ClientError::Message("invalid daemon response".into()))
            },
            _ => Err(ClientError::Message("invalid daemon response".into()))
        }
    }

    pub async fn quick_search<Q: AsRef<str>>(&mut self, query: Q) -> ClientResult<Vec<api::QuickSearchEntry>> {
        self.stream.write_u8(ipc::QUICK_SEARCH).await.map_err(ClientError::IO)?;
        ipc::write_string(&mut self.stream, query).await.map_err(ClientError::IO)?;
        self.stream.flush().await.map_err(ClientError::IO)?;

        match self.read_response().await? {
            ipc::QUICK_SEARCH_RESULT => match self.read_response().await? {
                ipc::QUICK_SEARCH_RESULT_OK => {
                    let number_of_results = self.stream.read_u64().await.map_err(ClientError::IO)?;

                    let mut results: Vec<api::QuickSearchEntry> = Vec::with_capacity(number_of_results as usize);

                    for _ in 0 .. number_of_results {
                        let id = self.stream.read_u64().await.map_err(ClientError::IO)?;
                        let title = ipc::read_string(&mut self.stream).await.map_err(ClientError::IO)?;

                        results.push(api::QuickSearchEntry { id, title });
                    }

                    Ok(results)
                },
                ipc::QUICK_SEARCH_RESULT_ERROR_REQUEST => Err(ClientError::Message("request error".into())),
                ipc::QUICK_SEARCH_RESULT_ERROR_NOT_LOGGED_IN => Err(ClientError::Message("you need to log in before using quick search".into())),
                _ => Err(ClientError::Message("invalid daemon response".into()))
            },
            _ => Err(ClientError::Message("invalid daemon response".into()))
        }
    }
}