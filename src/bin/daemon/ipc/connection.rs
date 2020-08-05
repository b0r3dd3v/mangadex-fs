use mangadex_fs::ipc;
use mangadex_fs::api;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

pub struct Connection {
    stream: tokio::net::UnixStream,
    api: std::sync::Arc<tokio::sync::Mutex<api::MangaDexAPI>>
}

pub enum ConnectionError<E> {
    IO(std::io::Error),
    API(E)
}

pub type ConnectionResult<R, E> = Result<R, ConnectionError<E>>;

impl Connection {
    pub fn new(stream: tokio::net::UnixStream, api: std::sync::Arc<tokio::sync::Mutex<api::MangaDexAPI>>) -> Connection {
        Connection {
            stream: stream,
            api: api
        }
    }

    pub async fn read_command(&mut self) -> std::io::Result<ipc::SubCommand> {
        self.stream.read_u8().await
    }

    pub async fn log_in(&mut self) -> ConnectionResult<(), api::LogInError> {
        let username = ipc::read_string(&mut self.stream).await.map_err(ConnectionError::IO)?;
        let password = ipc::read_string(&mut self.stream).await.map_err(ConnectionError::IO)?;

        debug!("attempting to log in with username \"{}\"...", username);

        match self.api.lock().await.log_in(username.clone(), password).await {
            Ok(session) => {
                info!("logged in successfully as {}", username);
                debug!("session id: {}", session.id);

                self.stream.write_u8(ipc::LOG_IN_RESULT).await.map_err(ConnectionError::IO)?;
                self.stream.write_u8(ipc::LOG_IN_RESULT_OK).await.map_err(ConnectionError::IO)?;
                self.stream.flush().await.map_err(ConnectionError::IO)?;

                Ok(())
            },
            Err(error) => {
                error!("log in error: {:?}", error);

                self.stream.write_u8(ipc::LOG_IN_RESULT).await.map_err(ConnectionError::IO)?;

                let error_byte = match error {
                    api::LogInError::Request(_) => ipc::LOG_IN_RESULT_ERROR_REQUEST,
                    api::LogInError::Invalid => ipc::LOG_IN_RESULT_ERROR_INVALID,
                    api::LogInError::Response(_) => ipc::LOG_IN_RESULT_ERROR_RESPONSE
                };

                self.stream.write_u8(error_byte).await.map_err(ConnectionError::IO)?;

                if let api::LogInError::Response(body) = &error {
                    ipc::write_string(&mut self.stream, body).await.map_err(ConnectionError::IO)?;
                }

                self.stream.flush().await.map_err(ConnectionError::IO)?;

                Err(ConnectionError::API(error))
            }
        }
    }

    pub async fn log_out(&mut self) -> ConnectionResult<(), api::LogOutError> {
        debug!("attempting to log out...");

        match self.api.lock().await.log_out().await {
            Ok(_) => {
                info!("logged out successfully");

                self.stream.write_u8(ipc::LOG_OUT_RESULT).await.map_err(ConnectionError::IO)?;
                self.stream.write_u8(ipc::LOG_OUT_RESULT_OK).await.map_err(ConnectionError::IO)?;
                self.stream.flush().await.map_err(ConnectionError::IO)?;

                Ok(())
            },
            Err(error) => {
                error!("log out error: {:?}", error);

                self.stream.write_u8(ipc::LOG_IN_RESULT).await.map_err(ConnectionError::IO)?;

                let error_byte = match error {
                    api::LogOutError::Request(_) => ipc::LOG_OUT_RESULT_ERROR_REQUEST,
                    api::LogOutError::Response(_) => ipc::LOG_OUT_RESULT_ERROR_RESPONSE
                };

                self.stream.write_u8(error_byte).await.map_err(ConnectionError::IO)?;

                if let api::LogOutError::Response(body) = &error {
                    ipc::write_string(&mut self.stream, body).await.map_err(ConnectionError::IO)?;
                }

                self.stream.flush().await.map_err(ConnectionError::IO)?;

                Err(ConnectionError::API(error))
            }
        }
    }

    pub async fn add_manga(&mut self) -> ConnectionResult<(), api::AddMangaError> {
        let manga_id = self.stream.read_u64().await.map_err(ConnectionError::IO)?;

        match self.api.lock().await.get_manga(manga_id).await {
            Ok(manga) => {
                info!("added manga: id: {}, title: {}", manga_id, manga.manga.title);

                self.stream.write_u8(ipc::ADD_MANGA_RESULT).await.map_err(ConnectionError::IO)?;
                self.stream.write_u8(ipc::ADD_MANGA_RESULT_OK).await.map_err(ConnectionError::IO)?;
                ipc::write_string(&mut self.stream, &manga.manga.title).await.map_err(ConnectionError::IO)?;
                self.stream.flush().await.map_err(ConnectionError::IO)?;

                Ok(())
            },
            Err(error) => {
                error!("add manga error: {}", error);

                self.stream.write_u8(ipc::ADD_MANGA_RESULT).await.map_err(ConnectionError::IO)?;
                self.stream.write_u8(ipc::ADD_MANGA_RESULT_ERROR_REQUEST).await.map_err(ConnectionError::IO)?;
                self.stream.flush().await.map_err(ConnectionError::IO)?;

                Err(ConnectionError::API(error))
            }
        }
    }

    pub async fn add_chapter(&mut self) -> ConnectionResult<(), api::AddChapterError> {
        let chapter_id = self.stream.read_u64().await.map_err(ConnectionError::IO)?;

        match self.api.lock().await.get_chapter(chapter_id).await {
            Ok(chapter) => {
                info!("added chapter: id: {}, manga_id: {}", chapter.id, chapter.manga_id);
                
                self.stream.write_u8(ipc::ADD_CHAPTER_RESULT).await.map_err(ConnectionError::IO)?;
                self.stream.write_u8(ipc::ADD_CHAPTER_RESULT_OK).await.map_err(ConnectionError::IO)?;
                self.stream.flush().await.map_err(ConnectionError::IO)?;

                Ok(())
            },
            Err(error) => {
                error!("add chapter error: {}", error);

                self.stream.write_u8(ipc::ADD_CHAPTER_RESULT).await.map_err(ConnectionError::IO)?;
                self.stream.write_u8(ipc::ADD_CHAPTER_RESULT_ERROR_REQUEST).await.map_err(ConnectionError::IO)?;
                self.stream.flush().await.map_err(ConnectionError::IO)?;

                Err(ConnectionError::API(error))
            }
        }
    }

    pub async fn quick_search(&mut self) -> ConnectionResult<(), api::QuickSearchError> {
        let query = ipc::read_string(&mut self.stream).await.map_err(ConnectionError::IO)?;

        match self.api.lock().await.quick_search(&query).await {
            Ok(results) => {
                info!("found {} results for quick search query \"{}\"", results.len(), query);

                self.stream.write_u8(ipc::QUICK_SEARCH_RESULT).await.map_err(ConnectionError::IO)?;
                self.stream.write_u8(ipc::QUICK_SEARCH_RESULT_OK).await.map_err(ConnectionError::IO)?;
                self.stream.write_u64(results.len() as u64).await.map_err(ConnectionError::IO)?;

                for result in results {
                    self.stream.write_u64(result.id).await.map_err(ConnectionError::IO)?;
                    ipc::write_string(&mut self.stream, result.title).await.map_err(ConnectionError::IO)?;
                }

                self.stream.flush().await.map_err(ConnectionError::IO)?;

                Ok(())
            },
            Err(error) => {
                error!("quick search error: {:?}", error);
                
                self.stream.write_u8(ipc::QUICK_SEARCH_RESULT).await.map_err(ConnectionError::IO)?;

                let error_byte = match error {
                    api::QuickSearchError::Request(_) => ipc::QUICK_SEARCH_RESULT_ERROR_REQUEST,
                    api::QuickSearchError::NotLoggedIn => ipc::QUICK_SEARCH_RESULT_ERROR_NOT_LOGGED_IN
                };

                self.stream.write_u8(error_byte).await.map_err(ConnectionError::IO)?;
                self.stream.flush().await.map_err(ConnectionError::IO)?;

                Err(ConnectionError::API(error))
            }
        }
    }
}