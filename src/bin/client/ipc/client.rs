use mangadex_fs::ipc;
use mangadex_fs::api;
use mangadex_fs::ipc::{IpcSend, IpcTryReceive};

pub struct Client {
    stream: tokio::net::UnixStream
}

#[derive(Debug)]
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

    pub async fn add_manga(&mut self, manga_id: u64, languages: Vec<String>) -> ClientResult<String> {
        ipc::Command::AddManga(manga_id, languages).ipc_send(&mut self.stream).await.map_err(ClientError::IO)?;

        match ipc::Response::ipc_try_receive(&mut self.stream).await.map_err(ClientError::IO)? {
            Some(ipc::Response::AddManga(Ok(formatted))) => Ok(formatted),
            Some(ipc::Response::AddManga(Err(failure))) => Err(ClientError::Message(failure)),
            _ => Err(ClientError::Message("unexpected daemon response".into()))
        }
    }

    pub async fn search(&mut self, params: api::SearchParams) -> ClientResult<Vec<api::SearchEntry>> {
        ipc::Command::Search(params).ipc_send(&mut self.stream).await.map_err(ClientError::IO)?;

        match ipc::Response::ipc_try_receive(&mut self.stream).await.map_err(ClientError::IO)? {
            Some(ipc::Response::Search(Ok(entries))) => Ok(entries),
            Some(ipc::Response::Search(Err(failure))) => Err(ClientError::Message(failure)),
            _ => Err(ClientError::Message("unexpected daemon response".into()))
        }
    }

    pub async fn mdlist(&mut self, params: api::MDListParams) -> ClientResult<Vec<api::MDListEntry>> {
        ipc::Command::MDList(params).ipc_send(&mut self.stream).await.map_err(ClientError::IO)?;

        match ipc::Response::ipc_try_receive(&mut self.stream).await.map_err(ClientError::IO)? {
            Some(ipc::Response::MDList(Ok(entries))) => Ok(entries),
            Some(ipc::Response::MDList(Err(failure))) => Err(ClientError::Message(failure)),
            _ => Err(ClientError::Message("unexpected daemon response".into()))
        }
    }

    pub async fn end_connection(mut self) -> ClientResult<()> {
        ipc::Command::EndConnection.ipc_send(&mut self.stream).await.map_err(ClientError::IO)
    }

    pub async fn follow_manga(&mut self, id: u64, status: api::MDListStatus) -> ClientResult<()> {
        ipc::Command::FollowManga(id, status).ipc_send(&mut self.stream).await.map_err(ClientError::IO)?;

        match ipc::Response::ipc_try_receive(&mut self.stream).await.map_err(ClientError::IO)? {
            Some(ipc::Response::FollowManga(Ok(_))) => Ok(()),
            Some(ipc::Response::FollowManga(Err(failure))) => Err(ClientError::Message(failure)),
            _ => Err(ClientError::Message("unexpected daemon response".into()))
        }
    }

    pub async fn unfollow_manga(&mut self, id: u64) -> ClientResult<()> {
        ipc::Command::UnfollowManga(id).ipc_send(&mut self.stream).await.map_err(ClientError::IO)?;

        match ipc::Response::ipc_try_receive(&mut self.stream).await.map_err(ClientError::IO)? {
            Some(ipc::Response::UnfollowManga(Ok(_))) => Ok(()),
            Some(ipc::Response::UnfollowManga(Err(failure))) => Err(ClientError::Message(failure)),
            _ => Err(ClientError::Message("unexpected daemon response".into()))
        }
    }
}