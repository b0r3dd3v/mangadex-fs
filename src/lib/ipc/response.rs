use crate::ipc;
use crate::api;
use ipc::{IpcReceive};
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
impl ipc::IpcTryReceive for api::MangaDexSession {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<api::MangaDexSession>> {
        Ok(Some(api::MangaDexSession {
            id: String::ipc_receive(stream).await?,
            remember_me_token: String::ipc_receive(stream).await?
        }))
    }
}

#[async_trait::async_trait]
impl ipc::IpcSend for api::SearchEntry {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        stream.write_u64(self.id).await?;
        self.title.ipc_send(stream).await?;
        self.author.ipc_send(stream).await?;
        let status = match &self.status {
            Some(status) => Some(match status {
                api::MDListStatus::Reading => 0u8,
                api::MDListStatus::Completed => 1u8,
                api::MDListStatus::OnHold => 2u8,
                api::MDListStatus::PlanToRead => 3u8,
                api::MDListStatus::Dropped => 4u8,
                api::MDListStatus::ReReading => 5u8
            }),
            None => None
        };

        status.ipc_send(stream).await?;
        self.last_update.ipc_send(stream).await
    }
}

#[async_trait::async_trait]
impl ipc::IpcTryReceive for api::SearchEntry {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<api::SearchEntry>> {
        let id = stream.read_u64().await?;
        let title = String::ipc_receive(stream).await?;
        let author = String::ipc_receive(stream).await?;

        let status = match Option::<u8>::ipc_try_receive(stream).await? {
            Some(status) => match status {
                Some(byte) => Some(match byte {
                    0u8 => api::MDListStatus::Reading,
                    1u8 => api::MDListStatus::Completed,
                    2u8 => api::MDListStatus::OnHold,
                    3u8 => api::MDListStatus::PlanToRead,
                    4u8 => api::MDListStatus::Dropped,
                    5u8 => api::MDListStatus::ReReading,
                    _ => return Ok(None)
                }),
                _ => None
            },
            None => return Ok(None)
        };

        let last_update = String::ipc_receive(stream).await?;

        Ok(Some(api::SearchEntry { id, title, author, status, last_update }))
    }
}


#[async_trait::async_trait]
impl ipc::IpcSend for api::MDListEntry {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        stream.write_u64(self.id).await?;
        self.title.ipc_send(stream).await?;
        self.author.ipc_send(stream).await?;

        let status = match &self.status {
            api::MDListStatus::Reading => 0u8,
            api::MDListStatus::Completed => 1u8,
            api::MDListStatus::OnHold => 2u8,
            api::MDListStatus::PlanToRead => 3u8,
            api::MDListStatus::Dropped => 4u8,
            api::MDListStatus::ReReading => 5u8
        };

        status.ipc_send(stream).await?;
        self.last_update.ipc_send(stream).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ipc::IpcTryReceive for api::MDListEntry {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<api::MDListEntry>> {
        let id = stream.read_u64().await?;
        let title = String::ipc_receive(stream).await?;
        let author = String::ipc_receive(stream).await?;

        let status = match u8::ipc_receive(stream).await? {
            0u8 => api::MDListStatus::Reading,
            1u8 => api::MDListStatus::Completed,
            2u8 => api::MDListStatus::OnHold,
            3u8 => api::MDListStatus::PlanToRead,
            4u8 => api::MDListStatus::Dropped,
            5u8 => api::MDListStatus::ReReading,
            _ => return Ok(None)
        };

        let last_update = String::ipc_receive(stream).await?;

        Ok(Some(api::MDListEntry { id, title, author, status, last_update }))
    }
}

#[async_trait::async_trait]
impl ipc::IpcSend for api::MDListNotLoggedInEntry {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        stream.write_u64(self.id).await?;
        self.title.ipc_send(stream).await?;

        let status = match &self.status {
            api::MDListStatus::Reading => 0u8,
            api::MDListStatus::Completed => 1u8,
            api::MDListStatus::OnHold => 2u8,
            api::MDListStatus::PlanToRead => 3u8,
            api::MDListStatus::Dropped => 4u8,
            api::MDListStatus::ReReading => 5u8
        };

        status.ipc_send(stream).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ipc::IpcTryReceive for api::MDListNotLoggedInEntry {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<api::MDListNotLoggedInEntry>> {
        let id = stream.read_u64().await?;
        let title = String::ipc_receive(stream).await?;

        let status = match u8::ipc_receive(stream).await? {
            0u8 => api::MDListStatus::Reading,
            1u8 => api::MDListStatus::Completed,
            2u8 => api::MDListStatus::OnHold,
            3u8 => api::MDListStatus::PlanToRead,
            4u8 => api::MDListStatus::Dropped,
            5u8 => api::MDListStatus::ReReading,
            _ => return Ok(None)
        };

        Ok(Some(api::MDListNotLoggedInEntry { id, title, status }))
    }
}

#[async_trait::async_trait]
impl ipc::IpcSend for api::MDList {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        match self {
            api::MDList::NotLoggedIn(vec) => {
                0u8.ipc_send(stream).await?;
                vec.ipc_send(stream).await
            },
            api::MDList::LoggedIn(vec) => {
                1u8.ipc_send(stream).await?;
                vec.ipc_send(stream).await
            }
        }
    }
}

#[async_trait::async_trait]
impl ipc::IpcTryReceive for api::MDList {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<api::MDList>> {
        Ok(match u8::ipc_receive(stream).await? {
            0u8 => Vec::<api::MDListNotLoggedInEntry>::ipc_try_receive(stream).await?.map(api::MDList::NotLoggedIn),
            1u8 => Vec::<api::MDListEntry>::ipc_try_receive(stream).await?.map(api::MDList::LoggedIn),
            _ => None
        })
    }
}

#[derive(Debug)]
pub enum Response {
    Kill,
    LogIn(Result<api::MangaDexSession, String>),
    LogOut(Result<(), String>),
    AddManga(Result<String, String>),
    Search(Result<Vec<api::SearchEntry>, String>),
    MDList(Result<api::MDList, String>)
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
            },
            Response::MDList(mdlist) => {
                stream.write_u8(ipc::RESPONSE_MDLIST).await?;
                mdlist.ipc_send(stream).await
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
            ipc::RESPONSE_MDLIST => Result::<api::MDList, String>::ipc_try_receive(stream).await?.map(Response::MDList),
            byte => {
                warn!("received unknown response byte: {}", byte);
                None
            }
        })
    }
}