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
                    ipc::Command::AddManga(id) => self.add_manga(id).await?,
                    ipc::Command::AddChapter(id) => self.add_chapter(id).await?,
                    ipc::Command::QuickSearch(query) => self.quick_search(query).await?
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

    pub async fn add_manga(&mut self, manga_id: u64) -> std::io::Result<ipc::Response> {
        Ok(match self.context.get_or_fetch_manga(manga_id).await {
            Ok(mangadex_fs::GetOrFetch::Cached(manga_ref)) => match manga_ref.upgrade() {
                Some(manga) => {
                    info!("cached manga {}: {:?}", manga_id, manga);

                    ipc::Response::AddManga(Ok(manga.title.clone()))
                },
                None => {
                    warn!("cached manga {}: pointer dropped?", manga_id);

                    ipc::Response::AddManga(Err("pointer dropped".into()))
                }
            },
            Ok(mangadex_fs::GetOrFetch::Fetched(manga_ref)) => match manga_ref.upgrade() {
                Some(manga) => {
                    info!("fetched manga {}: {:?}", manga_id, manga);

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

    pub async fn add_chapter(&mut self, chapter_id: u64) -> std::io::Result<ipc::Response> {
        Ok(match self.context.get_or_fetch_chapter(chapter_id).await {
            Ok(mangadex_fs::GetOrFetch::Cached(chapter_ref)) => match chapter_ref.upgrade() {
                Some(chapter) => {
                    let formatted = match (chapter.title.is_empty(), chapter.volume.is_empty()) {
                        (true, true) => format!("Ch. {}", chapter.chapter),
                        (true, false) => format!("Vol. {} Ch. {}", chapter.volume, chapter.chapter),
                        (false, true) => format!("Ch. {} - {}", chapter.chapter, chapter.title),
                        (false, false) => format!("Vol. {} Ch. {} - {}", chapter.volume, chapter.chapter, chapter.title)
                    };

                    info!("cached chapter {}: {}", chapter_id, formatted);

                    ipc::Response::AddChapter(Ok(formatted))
                },
                None => {
                    warn!("cached chapter {}: pointer dropped?", chapter_id);

                    ipc::Response::AddChapter(Err("pointer dropped".into()))
                }
            },
            Ok(mangadex_fs::GetOrFetch::Fetched(chapter_ref)) => match chapter_ref.upgrade() {
                Some(chapter) => {
                    let formatted = match (chapter.title.is_empty(), chapter.volume.is_empty()) {
                        (true, true) => format!("Ch. {}", chapter.chapter),
                        (true, false) => format!("Vol. {} Ch. {}", chapter.volume, chapter.chapter),
                        (false, true) => format!("Ch. {} - {}", chapter.chapter, chapter.title),
                        (false, false) => format!("Vol. {} Ch. {} - {}", chapter.volume, chapter.chapter, chapter.title)
                    };

                    info!("fetched chapter {}: {}", chapter_id, formatted);

                    ipc::Response::AddChapter(Ok(formatted))
                },
                None => {
                    warn!("fetched chapter {}: pointer dropped?", chapter_id);

                    ipc::Response::AddChapter(Err("pointer dropped".into()))
                }
            },
            Err(error) => {
                error!("add chapter request error: {}", error);

                ipc::Response::AddChapter(Err("request error".into()))
            }
        })
    }

    pub async fn quick_search(&mut self, query: String) -> std::io::Result<ipc::Response> {
        Ok(match self.context.quick_search(&query).await {
            Ok(results) => {
                info!("found {} results for quick search query \"{}\"", results.len(), query);

                ipc::Response::QuickSearch(Ok(results))
            },
            Err(error) => {
                error!("quick search error: {:?}", error);
                
                match error {
                    api::QuickSearchError::Request(_) => ipc::Response::QuickSearch(Err("request error".into())),
                    api::QuickSearchError::NotLoggedIn => ipc::Response::QuickSearch(Err("you need to be logged in to use quick search".into()))
                }
            }
        })
    }
}