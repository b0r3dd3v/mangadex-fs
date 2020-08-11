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
/*

    pub manga_id: u64,
    pub manga_title: String,
    pub chapter_id: u64,
    pub chapter: String,
    pub chapter_title: String,
    pub marked_read: bool,
    pub last_update: String
    */
#[async_trait::async_trait]
impl ipc::IpcSend for api::FollowsEntry {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        stream.write_u64(self.manga_id).await?;
        self.manga_title.ipc_send(stream).await?;
        stream.write_u64(self.chapter_id).await?;
        self.chapter.ipc_send(stream).await?;
        self.chapter_title.ipc_send(stream).await?;
        self.chapter_volume.ipc_send(stream).await?;
        if self.marked_read {
            stream.write_u8(1u8).await?;
        }
        else {
            stream.write_u8(0u8).await?;
        }
        self.last_update.ipc_send(stream).await
    }
}

#[async_trait::async_trait]
impl ipc::IpcTryReceive for api::FollowsEntry {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<api::FollowsEntry>> {
        let manga_id = stream.read_u64().await?;
        let manga_title = String::ipc_receive(stream).await?;
        let chapter_id = stream.read_u64().await?;
        let chapter = String::ipc_receive(stream).await?;
        let chapter_title = String::ipc_receive(stream).await?;
        let chapter_volume = String::ipc_receive(stream).await?;
        let marked_read = match stream.read_u8().await? {
            0u8 => false,
            1u8 => true,
            _ => return Ok(None)
        };
        let last_update = String::ipc_receive(stream).await?;

        Ok(Some(api::FollowsEntry {
            manga_id, manga_title, chapter_id, chapter, chapter_title, chapter_volume, marked_read, last_update
        }))
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
                api::MDListStatus::Reading => ipc::MDLIST_STATUS_READING,
                api::MDListStatus::Completed => ipc::MDLIST_STATUS_COMPLETED,
                api::MDListStatus::OnHold => ipc::MDLIST_STATUS_ON_HOLD,
                api::MDListStatus::PlanToRead => ipc::MDLIST_STATUS_PLAN_TO_READ,
                api::MDListStatus::Dropped => ipc::MDLIST_STATUS_DROPPED,
                api::MDListStatus::ReReading => ipc::MDLIST_STATUS_RE_READING
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
                    ipc::MDLIST_STATUS_READING => api::MDListStatus::Reading,
                    ipc::MDLIST_STATUS_COMPLETED => api::MDListStatus::Completed,
                    ipc::MDLIST_STATUS_ON_HOLD => api::MDListStatus::OnHold,
                    ipc::MDLIST_STATUS_PLAN_TO_READ => api::MDListStatus::PlanToRead,
                    ipc::MDLIST_STATUS_DROPPED => api::MDListStatus::Dropped,
                    ipc::MDLIST_STATUS_RE_READING => api::MDListStatus::ReReading,
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
            api::MDListStatus::Reading => ipc::MDLIST_STATUS_READING,
            api::MDListStatus::Completed => ipc::MDLIST_STATUS_COMPLETED,
            api::MDListStatus::OnHold => ipc::MDLIST_STATUS_ON_HOLD,
            api::MDListStatus::PlanToRead => ipc::MDLIST_STATUS_PLAN_TO_READ,
            api::MDListStatus::Dropped => ipc::MDLIST_STATUS_DROPPED,
            api::MDListStatus::ReReading => ipc::MDLIST_STATUS_RE_READING
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
            ipc::MDLIST_STATUS_READING => api::MDListStatus::Reading,
            ipc::MDLIST_STATUS_COMPLETED => api::MDListStatus::Completed,
            ipc::MDLIST_STATUS_ON_HOLD => api::MDListStatus::OnHold,
            ipc::MDLIST_STATUS_PLAN_TO_READ => api::MDListStatus::PlanToRead,
            ipc::MDLIST_STATUS_DROPPED => api::MDListStatus::Dropped,
            ipc::MDLIST_STATUS_RE_READING => api::MDListStatus::ReReading,
            _ => return Ok(None)
        };

        let last_update = String::ipc_receive(stream).await?;

        Ok(Some(api::MDListEntry { id, title, author, status, last_update }))
    }
}

#[derive(Debug)]
pub enum Response {
    Kill,
    LogIn(Result<api::MangaDexSession, String>),
    LogOut(Result<(), String>),
    AddManga(Result<String, String>),
    Search(Result<Vec<api::SearchEntry>, String>),
    MDList(Result<Vec<api::MDListEntry>, String>),
    FollowManga(Result<(), String>),
    UnfollowManga(Result<(), String>),
    MarkChapterRead(Result<(), String>),
    MarkChapterUnread(Result<(), String>),
    Follows(Result<Vec<api::FollowsEntry>, String>)
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
            Response::FollowManga(followmanga) => {
                stream.write_u8(ipc::RESPONSE_FOLLOW_MANGA).await?;
                followmanga.ipc_send(stream).await
            },
            Response::UnfollowManga(unfollowmanga) => {
                stream.write_u8(ipc::RESPONSE_UNFOLLOW_MANGA).await?;
                unfollowmanga.ipc_send(stream).await
            },
            Response::MarkChapterRead(markchapterread) => {
                stream.write_u8(ipc::RESPONSE_MARK_CHAPTER_READ).await?;
                markchapterread.ipc_send(stream).await
            },
            Response::MarkChapterUnread(markchapterunread) => {
                stream.write_u8(ipc::RESPONSE_MARK_CHAPTER_UNREAD).await?;
                markchapterunread.ipc_send(stream).await
            },
            Response::Follows(follows) => {
                stream.write_u8(ipc::RESPONSE_FOLLOWS).await?;
                follows.ipc_send(stream).await
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
            ipc::RESPONSE_MDLIST => Result::<Vec<api::MDListEntry>, String>::ipc_try_receive(stream).await?.map(Response::MDList),
            ipc::RESPONSE_FOLLOW_MANGA => Result::<(), String>::ipc_try_receive(stream).await?.map(Response::FollowManga),
            ipc::RESPONSE_UNFOLLOW_MANGA => Result::<(), String>::ipc_try_receive(stream).await?.map(Response::UnfollowManga),
            ipc::RESPONSE_MARK_CHAPTER_READ => Result::<(), String>::ipc_try_receive(stream).await?.map(Response::MarkChapterRead),
            ipc::RESPONSE_MARK_CHAPTER_UNREAD => Result::<(), String>::ipc_try_receive(stream).await?.map(Response::MarkChapterUnread),
            ipc::RESPONSE_FOLLOWS => Result::<Vec<api::FollowsEntry>, String>::ipc_try_receive(stream).await?.map(Response::Follows),
            byte => {
                warn!("received unknown response byte: {}", byte);
                None
            }
        })
    }
}