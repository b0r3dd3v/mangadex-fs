use mangadex_fs::ipc;
use mangadex_fs::api;
use mangadex_fs::ipc::{IpcSend, IpcTryReceive};

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

    pub async fn kill(&mut self) -> ClientResult<()> {
        ipc::Command::Kill.ipc_send(&mut self.stream).await.map_err(ClientError::IO)?;

        match ipc::Response::ipc_try_receive(&mut self.stream).await.map_err(ClientError::IO)? {
            Some(ipc::Response::Kill) => Ok(()),
            _ => Err(ClientError::Message("unexpected daemon response".into()))
        }
    }

    pub async fn log_in<U: Into<String>, P: Into<String>>(&mut self, username: U, password: P) -> ClientResult<api::MangaDexSession> {
        ipc::Command::LogIn(username.into(), password.into()).ipc_send(&mut self.stream).await.map_err(ClientError::IO)?;

        match ipc::Response::ipc_try_receive(&mut self.stream).await.map_err(ClientError::IO)? {
            Some(ipc::Response::LogIn(Ok(session))) => Ok(session),
            Some(ipc::Response::LogIn(Err(failure))) => Err(ClientError::Message(failure)),
            _ => Err(ClientError::Message("unexpected daemon response".into()))
        }
    }

    pub async fn log_out(&mut self) -> ClientResult<()> {
        ipc::Command::LogOut.ipc_send(&mut self.stream).await.map_err(ClientError::IO)?;
 
        match ipc::Response::ipc_try_receive(&mut self.stream).await.map_err(ClientError::IO)? {
            Some(ipc::Response::LogOut(Ok(_))) => Ok(()),
            Some(ipc::Response::LogOut(Err(failure))) => Err(ClientError::Message(failure)),
            _ => Err(ClientError::Message("unexpected daemon response".into()))
        }
    }

    pub async fn add_manga(&mut self, manga_id: u64) -> ClientResult<String> {
        ipc::Command::AddManga(manga_id).ipc_send(&mut self.stream).await.map_err(ClientError::IO)?;

        match ipc::Response::ipc_try_receive(&mut self.stream).await.map_err(ClientError::IO)? {
            Some(ipc::Response::AddManga(Ok(formatted))) => Ok(formatted),
            Some(ipc::Response::AddManga(Err(failure))) => Err(ClientError::Message(failure)),
            _ => Err(ClientError::Message("unexpected daemon response".into()))
        }
    }

    pub async fn add_chapter(&mut self, chapter_id: u64) -> ClientResult<String> {
        ipc::Command::AddChapter(chapter_id).ipc_send(&mut self.stream).await.map_err(ClientError::IO)?;

        match ipc::Response::ipc_try_receive(&mut self.stream).await.map_err(ClientError::IO)? {
            Some(ipc::Response::AddChapter(Ok(formatted))) => Ok(formatted),
            Some(ipc::Response::AddChapter(Err(failure))) => Err(ClientError::Message(failure)),
            _ => Err(ClientError::Message("unexpected daemon response".into()))
        }
    }

    pub async fn quick_search<Q: Into<String>>(&mut self, query: Q) -> ClientResult<Vec<api::QuickSearchEntry>> {
        ipc::Command::QuickSearch(query.into()).ipc_send(&mut self.stream).await.map_err(ClientError::IO)?;

        match ipc::Response::ipc_try_receive(&mut self.stream).await.map_err(ClientError::IO)? {
            Some(ipc::Response::QuickSearch(Ok(entries))) => Ok(entries),
            Some(ipc::Response::QuickSearch(Err(failure))) => Err(ClientError::Message(failure)),
            _ => Err(ClientError::Message("unexpected daemon response".into()))
        }
    }
}