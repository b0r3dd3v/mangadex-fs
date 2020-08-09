use crate::ipc;
use crate::api;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

#[async_trait::async_trait]
impl ipc::IpcSend for api::MangaDexSession {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        self.id.ipc_send(stream).await?;
        self.remember_me_token.ipc_send(stream).await
    }
}

#[async_trait::async_trait]
impl ipc::IpcReceive for api::MangaDexSession {
    async fn ipc_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<api::MangaDexSession> {
        Ok(api::MangaDexSession {
            id: String::ipc_receive(stream).await?,
            remember_me_token: String::ipc_receive(stream).await?
        })
    }
}

#[async_trait::async_trait]
impl ipc::IpcSend for api::SearchEntry {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        stream.write_u64(self.id).await?;
        self.title.ipc_send(stream).await
    }
}

#[async_trait::async_trait]
impl ipc::IpcReceive for api::SearchEntry {
    async fn ipc_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<api::SearchEntry> {
        Ok(api::SearchEntry {
            id: stream.read_u64().await?,
            title: String::ipc_receive(stream).await?
        })
    }
}


#[derive(Debug)]
pub enum Response {
    Kill,
    LogIn(Result<api::MangaDexSession, String>),
    LogOut(Result<(), String>),
    AddManga(Result<String, String>),
    Search(Result<Vec<api::SearchEntry>, String>)
}

#[async_trait::async_trait]
impl ipc::IpcSend for Response {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        debug!("sending response: {:?}", self);

        match self {
            Response::Kill => stream.write_u8(ipc::RESPONSE_KILL).await,
            Response::LogIn(login) => {
                stream.write_u8(ipc::RESPONSE_LOG_IN).await?;
                login.ipc_send(stream).await
            },
            Response::LogOut(logout) => {
                stream.write_u8(ipc::RESPONSE_LOG_OUT).await?;
                logout.ipc_send(stream).await
            },
            Response::AddManga(addmanga) => {
                stream.write_u8(ipc::RESPONSE_ADD_MANGA).await?;
                addmanga.ipc_send(stream).await
            },
            Response::Search(search) => {
                stream.write_u8(ipc::RESPONSE_SEARCH).await?;
                search.ipc_send(stream).await
            }
        }
    }
}

#[async_trait::async_trait]
impl ipc::IpcTryReceive for Response {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<Self>> {
        Ok(match stream.read_u8().await? {
            ipc::RESPONSE_KILL => Some(Response::Kill),
            ipc::RESPONSE_LOG_IN => Result::<api::MangaDexSession, String>::ipc_try_receive(stream).await?.map(Response::LogIn),
            ipc::RESPONSE_LOG_OUT => Result::<(), String>::ipc_try_receive(stream).await?.map(Response::LogOut),
            ipc::RESPONSE_ADD_MANGA => Result::<String, String>::ipc_try_receive(stream).await?.map(Response::AddManga),
            ipc::RESPONSE_SEARCH => Result::<Vec<api::SearchEntry>, String>::ipc_try_receive(stream).await?.map(Response::Search),
            byte => {
                warn!("received unknown response byte: {}", byte);
                None
            }
        })
    }
}