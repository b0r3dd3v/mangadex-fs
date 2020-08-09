pub mod command;
pub mod response;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

pub use command::Command;
pub use response::Response;

use crate::ipc;

#[async_trait::async_trait]
pub trait IpcSend: std::marker::Sync + std::marker::Send {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()>;
}

#[async_trait::async_trait]
pub trait IpcReceive: Sized {
    async fn ipc_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Self>;
}

#[async_trait::async_trait]
pub trait IpcTryReceive: Sized {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<Self>>;
}

#[async_trait::async_trait]
impl<T: IpcReceive> IpcTryReceive for T {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<Self>> { Ok(Some(T::ipc_receive(stream).await?)) }
}

#[async_trait::async_trait]
impl IpcSend for () {
    async fn ipc_send(&self, _stream: &mut tokio::net::UnixStream) -> std::io::Result<()> { Ok(()) }
}

#[async_trait::async_trait]
impl IpcReceive for () {
    async fn ipc_receive(_stream: &mut tokio::net::UnixStream) -> std::io::Result<Self> { Ok(()) }
}

#[async_trait::async_trait]
impl IpcSend for String {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        let bytes: &[u8] = self.as_ref();

        stream.write_u64(bytes.len() as u64).await?;
        stream.write_all(&bytes).await
    }
}

#[async_trait::async_trait]
impl IpcReceive for String {
    async fn ipc_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Self> {
        let length: u64 = stream.read_u64().await?;
        let mut buffer = Vec::with_capacity(length as usize);
                            
        stream.read_buf(&mut buffer).await?;
    
        Ok(unsafe { String::from_utf8_unchecked(buffer) })
    }
}

#[async_trait::async_trait]
impl<T: IpcSend> IpcSend for Vec<T> {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        stream.write_u64(self.len() as u64).await?;

        for item in self.iter() {
            item.ipc_send(stream).await?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl<T: IpcReceive + Send> IpcReceive for Vec<T> {
    async fn ipc_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Self> {
        let length: u64 = stream.read_u64().await?;
        let mut buffer = Vec::with_capacity(length as usize);
                            
        for _ in 0 .. length {
            buffer.push(T::ipc_receive(stream).await?);
        }

        Ok(buffer)
    }
}

#[async_trait::async_trait]
impl<T: ipc::IpcSend, E: ipc::IpcSend> ipc::IpcSend for Result<T, E> {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        match self {
            Ok(success) => {
                stream.write_u8(0u8).await?;
                success.ipc_send(stream).await
            },
            Err(failure) => {
                stream.write_u8(1u8).await?;
                failure.ipc_send(stream).await
            }
        }
    }
}

#[async_trait::async_trait]
impl<T: ipc::IpcTryReceive, E: ipc::IpcTryReceive> ipc::IpcTryReceive for Result<T, E> {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<Self>> {
        Ok(match stream.read_u8().await? {
            0u8 => T::ipc_try_receive(stream).await?.map(|value| Ok(value)),
            1u8 => E::ipc_try_receive(stream).await?.map(|error| Err(error)),
            byte => {
                warn!("read invalid result byte: {}", byte);
                None
            }
        })
    }
}