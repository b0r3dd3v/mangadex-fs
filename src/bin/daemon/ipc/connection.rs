use mangadex_fs::ipc;
use mangadex_fs::api;

use mangadex_fs::ipc::{IpcSend, IpcTryReceive};

pub struct Connection {
    stream: tokio::net::UnixStream,
    context: std::sync::Arc<mangadex_fs::Context>,
    kill: tokio::sync::mpsc::Sender<()>
}

impl Connection {
    pub fn new(
        stream: tokio::net::UnixStream,
        context: std::sync::Arc<mangadex_fs::Context>,
        kill: tokio::sync::mpsc::Sender<()>
    ) -> Connection {
        Connection { stream, context, kill }
    }

    pub async fn handle(&mut self) -> std::io::Result<()> {
        match ipc::Command::ipc_try_receive(&mut self.stream).await? {
            Some(command) => {
                let response = match command {
                    ipc::Command::Kill => self.kill().await?,
                    ipc::Command::LogIn(username, password) => self.log_in(username, password).await?,
                    ipc::Command::LogOut => self.log_out().await?,
                    ipc::Command::AddManga(id, languages) => self.add_manga(id, languages).await?,
                    ipc::Command::Search(params) => self.search(&params).await?
                };

                response.ipc_send(&mut self.stream).await?;

                Ok(())
            },
            None => Ok(())
        }
    }

    pub async fn kill(&mut self) -> std::io::Result<ipc::Response> {
        self.kill.send(()).await.expect("MikuDex");

        Ok(ipc::Response::Kill)
    }

    pub async fn log_in(&mut self, username: String, password: String) -> std::io::Result<ipc::Response> {
        Ok(match self.context.log_in(username.clone(), password).await {
            Ok(session) => {
                info!("logged in successfully as {}", username);
                debug!("session id: {}", session.id);

                ipc::Response::LogIn(Ok(session.clone()))
            },
            Err(error) => {
                error!("log in error: {:?}", error);

                match error {
                    api::LogInError::Response(body) => ipc::Response::LogIn(Err(String::from("MangaDex response: ") + &body)),
                    _ => ipc::Response::LogIn(Err("request error".into()))
                }
            }
        })
    }

    pub async fn log_out(&mut self) -> std::io::Result<ipc::Response> {
        Ok(match self.context.log_out().await {
            Ok(_) => {
                info!("logged out successfully");

                ipc::Response::LogOut(Ok(()))
            },
            Err(error) => {
                error!("log out error: {:?}", error);

                ipc::Response::LogOut(Err("request error".into()))
            }
        })
    }

    pub async fn add_manga(&mut self, manga_id: u64, languages: Vec<String>) -> std::io::Result<ipc::Response> {
        Ok(match self.context.get_or_fetch_manga(manga_id, languages).await {
            Ok(mangadex_fs::GetOrFetch::Cached(manga_ref)) => match manga_ref.upgrade() {
                Some(manga) => {
                    info!("cached manga {}: {:?}", manga_id, manga.display());

                    ipc::Response::AddManga(Ok(manga.title.clone()))
                },
                None => {
                    warn!("cached manga {}: pointer dropped?", manga_id);

                    ipc::Response::AddManga(Err("pointer dropped".into()))
                }
            },
            Ok(mangadex_fs::GetOrFetch::Fetched(manga_ref)) => match manga_ref.upgrade() {
                Some(manga) => {
                    info!("fetched manga {}: {:?}", manga_id, manga.display());

                    ipc::Response::AddManga(Ok(manga.title.clone()))
                },
                None => {
                    warn!("fetched manga {}: pointer dropped?", manga_id);

                    ipc::Response::AddManga(Err("pointer dropped".into()))
                }
            },
            Err(error) => {
                error!("add manga request error: {}", error);

                ipc::Response::AddManga(Err("request error".into()))
            }
        })
    }

    pub async fn search(&mut self, params: &api::SearchParams) -> std::io::Result<ipc::Response> {
        Ok(match self.context.search(params).await {
            Ok(results) => {
                info!("found {} results for search query {:?}", results.len(), params);

                ipc::Response::Search(Ok(results))
            },
            Err(error) => {
                error!("search error: {:?}", error);
                
                match error {
                    api::SearchError::Request(_) => ipc::Response::Search(Err("request error".into())),
                    api::SearchError::NotLoggedIn => ipc::Response::Search(Err("you need to be logged in to use search".into()))
                }
            }
        })
    }
}