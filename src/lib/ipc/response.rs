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
impl ipc::IpcSend for api::QuickSearchEntry {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        stream.write_u64(self.id).await?;
        self.title.ipc_send(stream).await
    }
}

#[async_trait::async_trait]
impl ipc::IpcReceive for api::QuickSearchEntry {
    async fn ipc_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<api::QuickSearchEntry> {
        Ok(api::QuickSearchEntry {
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
    AddChapter(Result<String, String>),
    QuickSearch(Result<Vec<api::QuickSearchEntry>, String>)
}

#[async_trait::async_trait]
impl ipc::IpcSend for Response {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        debug!("sending response: {:?}", self);

        match self {
            Response::Kill => stream.write_u8(1u8).await,
            Response::LogIn(login) => {
                stream.write_u8(2u8).await?;
                login.ipc_send(stream).await
            },
            Response::LogOut(logout) => {
                stream.write_u8(3u8).await?;
                logout.ipc_send(stream).await
            },
            Response::AddManga(addmanga) => {
                stream.write_u8(4u8).await?;
                addmanga.ipc_send(stream).await
            },
            Response::AddChapter(addchapter) => {
                stream.write_u8(5u8).await?;
                addchapter.ipc_send(stream).await
            },
            Response::QuickSearch(quicksearch) => {
                stream.write_u8(6u8).await?;
                quicksearch.ipc_send(stream).await
            }
        }
    }
}

#[async_trait::async_trait]
impl ipc::IpcTryReceive for Response {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<Self>> {
        Ok(match stream.read_u8().await? {
            1u8 => Some(Response::Kill),
            2u8 => Result::<api::MangaDexSession, String>::ipc_try_receive(stream).await?.map(Response::LogIn),
            3u8 => Result::<(), String>::ipc_try_receive(stream).await?.map(Response::LogOut),
            4u8 => Result::<String, String>::ipc_try_receive(stream).await?.map(Response::AddManga),
            5u8 => Result::<String, String>::ipc_try_receive(stream).await?.map(Response::AddChapter),
            6u8 => Result::<Vec<api::QuickSearchEntry>, String>::ipc_try_receive(stream).await?.map(Response::QuickSearch),
            byte => {
                warn!("received unknown response byte: {}", byte);
                None
            }
        })
    }
}