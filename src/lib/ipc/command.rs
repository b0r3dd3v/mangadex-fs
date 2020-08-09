use crate::ipc;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use ipc::{IpcReceive};

#[derive(Debug)]
pub enum Command {
    Kill,
    LogIn(String, String),
    LogOut,
    AddManga(u64),
    AddChapter(u64),
    QuickSearch(String)
}

#[async_trait::async_trait]
impl ipc::IpcSend for Command {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        debug!("sending command: {:?}", self);

        match self {
            Command::Kill => stream.write_u8(1u8).await,
            Command::LogIn(username, password) => {
                stream.write_u8(2u8).await?;
                username.ipc_send(stream).await?;
                password.ipc_send(stream).await
            },
            Command::LogOut => stream.write_u8(3u8).await,
            Command::AddManga(id) => {
                stream.write_u8(4u8).await?;
                stream.write_u64(*id).await
            },
            Command::AddChapter(id) => {
                stream.write_u8(5u8).await?;
                stream.write_u64(*id).await
            },
            Command::QuickSearch(query) => {
                stream.write_u8(6u8).await?;
                query.ipc_send(stream).await
            }
        }
    }
}

#[async_trait::async_trait]
impl ipc::IpcTryReceive for Command {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<Self>> {
        Ok(match stream.read_u8().await? {
            1u8 => Some(Command::Kill),
            2u8 => Some(Command::LogIn(String::ipc_receive(stream).await?, String::ipc_receive(stream).await?)),
            3u8 => Some(Command::LogOut),
            4u8 => Some(Command::AddManga(stream.read_u64().await?)),
            5u8 => Some(Command::AddChapter(stream.read_u64().await?)),
            6u8 => Some(Command::QuickSearch(String::ipc_receive(stream).await?)),
            byte => {
                warn!("received unknown command byte: {}", byte);
                None
            }
        })
    }
}