use crate::ipc;
use crate::api;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use ipc::{IpcReceive};
use std::convert::TryFrom;

#[async_trait::async_trait]
impl ipc::IpcSend for api::MDListParams {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        stream.write_u64(self.id).await?;
        self.sort_by.encode().ipc_send(stream).await
    }
}

#[async_trait::async_trait]
impl ipc::IpcTryReceive for api::MDListParams {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<Self>> {
        let mut params = api::MDListParams::default();

        params.id = stream.read_u64().await?;
        params.sort_by =  match api::SortBy::decode(u8::ipc_receive(stream).await?) {
            Some(sort_by) => sort_by,
            None => return Ok(None)
        };

        Ok(Some(params))
    }
}

#[async_trait::async_trait]
impl ipc::IpcSend for api::SearchParams {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        self.title.ipc_send(stream).await?;
        self.author.ipc_send(stream).await?;
        self.artist.ipc_send(stream).await?;
        self.original_language.map(|lang| lang.code()).ipc_send(stream).await?;
        
        let mut demo_pub_bits: u8 = 0b00000000;

        if self.demographic.shounen { demo_pub_bits |=   0b00000001; }
        if self.demographic.shoujo { demo_pub_bits |=    0b00000010; }
        if self.demographic.seinen { demo_pub_bits |=    0b00000100; }
        if self.demographic.josei { demo_pub_bits |=     0b00001000; }
        if self.publication.ongoing { demo_pub_bits |=   0b00010000; }
        if self.publication.completed { demo_pub_bits |= 0b00100000; }
        if self.publication.cancelled { demo_pub_bits |= 0b01000000; }
        if self.publication.hiatus { demo_pub_bits |=    0b10000000; }

        demo_pub_bits.ipc_send(stream).await?;

        self.include_tag.iter().map(|genre| genre.clone() as u8).collect::<Vec<_>>().ipc_send(stream).await?;
        self.exclude_tag.iter().map(|genre| genre.clone() as u8).collect::<Vec<_>>().ipc_send(stream).await?;

        let tag_mode: u8 = match (&self.inclusion_mode, &self.exclusion_mode) {
            (api::TagMode::All, api::TagMode::All) => 0u8,
            (api::TagMode::All, api::TagMode::Any) => 1u8,
            (api::TagMode::Any, api::TagMode::All) => 2u8,
            (api::TagMode::Any, api::TagMode::Any) => 3u8,
        };

        tag_mode.ipc_send(stream).await?;

        self.sort_by.encode().ipc_send(stream).await
    }
}

#[async_trait::async_trait]
impl ipc::IpcTryReceive for api::SearchParams {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<Self>> {
        debug!("params");
        let mut params = api::SearchParams::default();

        params.title = String::ipc_receive(stream).await?;
        debug!("title: {}", params.title);
        params.author = match Option::<String>::ipc_try_receive(stream).await? {
            Some(author) => author,
            None => return Ok(None)
        };
        debug!("author: {:?}", params.author);
        params.artist = match Option::<String>::ipc_try_receive(stream).await? {
            Some(artist) => artist,
            None => return Ok(None)
        };
        debug!("artist: {:?}", params.artist);
        params.original_language = match Option::<u8>::ipc_try_receive(stream).await? {
            Some(maybe_byte) => match maybe_byte {
                Some(byte) => match api::Language::from_code(byte) {
                    Some(language) => Some(language),
                    None => return Ok(None)
                },
                None => None
            },
            None => return Ok(None)
        };
        debug!("language: {:?}", params.original_language);

        let demo_pub_bits = u8::ipc_receive(stream).await?;
        params.demographic.shounen = demo_pub_bits &   0b00000001 != 0;
        params.demographic.shoujo = demo_pub_bits &    0b00000010 != 0;
        params.demographic.seinen = demo_pub_bits &    0b00000100 != 0;
        params.demographic.josei = demo_pub_bits &     0b00001000 != 0;
        params.publication.ongoing = demo_pub_bits &   0b00010000 != 0;
        params.publication.completed = demo_pub_bits & 0b00100000 != 0;
        params.publication.cancelled = demo_pub_bits & 0b01000000 != 0;
        params.publication.hiatus = demo_pub_bits &    0b10000000 != 0;

        debug!("demographic: {:?}", params.demographic);
        debug!("publication: {:?}", params.publication);

        params.include_tag = {
            let bytes: Vec<u8> = Vec::ipc_receive(stream).await?;

            let mut tags: Vec<api::Genre> = vec![];

            for byte in bytes {
                match api::Genre::try_from(byte) {
                    Ok(tag) => tags.push(tag),
                    Err(_) => return Ok(None)
                }
            }

            tags
        };

        debug!("include: {:?}", params.include_tag);

        params.exclude_tag = {
            let bytes: Vec<u8> = Vec::ipc_receive(stream).await?;

            let mut tags: Vec<api::Genre> = vec![];

            for byte in bytes {
                match api::Genre::try_from(byte) {
                    Ok(tag) => tags.push(tag),
                    Err(_) => return Ok(None)
                }
            }

            tags
        };

        debug!("exclude: {:?}", params.exclude_tag);

        let (inclusion_mode, exclusion_mode) = match u8::ipc_receive(stream).await? {
            0u8 => (api::TagMode::All, api::TagMode::All),
            1u8 => (api::TagMode::All, api::TagMode::Any),
            2u8 => (api::TagMode::Any, api::TagMode::All),
            3u8 => (api::TagMode::Any, api::TagMode::Any),
            _ => return Ok(None)
        };

        params.inclusion_mode = inclusion_mode;
        params.exclusion_mode = exclusion_mode;
        debug!("mode: {:?} {:?}", params.inclusion_mode, params.exclusion_mode);

        params.sort_by = match api::SortBy::decode(u8::ipc_receive(stream).await?) {
            Some(sort_by) => sort_by,
            None => return Ok(None)
        };

        Ok(Some(params))
    }
}

#[derive(Debug)]
pub enum Command {
    EndConnection,
    Kill,
    LogIn(String, String),
    LogOut,
    AddManga(u64, Vec<String>),
    Search(api::SearchParams),
    MDList(api::MDListParams)
}

#[async_trait::async_trait]
impl ipc::IpcSend for Command {
    async fn ipc_send(&self, stream: &mut tokio::net::UnixStream) -> std::io::Result<()> {
        debug!("sending command: {:?}", self);

        match self {
            Command::EndConnection =>  stream.write_u8(ipc::COMMAND_END_CONNECTION).await,
            Command::Kill => stream.write_u8(ipc::COMMAND_KILL).await,
            Command::LogIn(username, password) => {
                stream.write_u8(ipc::COMMAND_LOG_IN).await?;
                username.ipc_send(stream).await?;
                password.ipc_send(stream).await
            },
            Command::LogOut => stream.write_u8(ipc::COMMAND_LOG_OUT).await,
            Command::AddManga(id, languages) => {
                stream.write_u8(ipc::COMMAND_ADD_MANGA).await?;
                stream.write_u64(*id).await?;
                languages.ipc_send(stream).await
            },
            Command::Search(params) => {
                stream.write_u8(ipc::COMMAND_SEARCH).await?;
                params.ipc_send(stream).await
            },
            Command::MDList(params) => {
                stream.write_u8(ipc::COMMAND_MDLIST).await?;
                params.ipc_send(stream).await
            }
        }
    }
}

#[async_trait::async_trait]
impl ipc::IpcTryReceive for Command {
    async fn ipc_try_receive(stream: &mut tokio::net::UnixStream) -> std::io::Result<Option<Self>> {
        Ok(match stream.read_u8().await? {
            ipc::COMMAND_END_CONNECTION => Some(Command::EndConnection),
            ipc::COMMAND_KILL => Some(Command::Kill),
            ipc::COMMAND_LOG_IN => Some(Command::LogIn(String::ipc_receive(stream).await?, String::ipc_receive(stream).await?)),
            ipc::COMMAND_LOG_OUT => Some(Command::LogOut),
            ipc::COMMAND_ADD_MANGA => Some(Command::AddManga(stream.read_u64().await?, Vec::<String>::ipc_receive(stream).await?)),
            ipc::COMMAND_SEARCH => api::SearchParams::ipc_try_receive(stream).await?.map(Command::Search),
            ipc::COMMAND_MDLIST => api::MDListParams::ipc_try_receive(stream).await?.map(Command::MDList),
            byte => {
                warn!("received unknown command byte: {}", byte);
                None
            }
        })
    }
}