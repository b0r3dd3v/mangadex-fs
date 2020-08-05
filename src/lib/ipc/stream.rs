use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

pub async fn read_string(stream: &mut tokio::net::UnixStream) -> std::io::Result<String> {
    let length: u64 = stream.read_u64().await?;
    let mut buffer = Vec::with_capacity(length as usize);
                        
    stream.read_buf(&mut buffer).await?;

    Ok(unsafe { String::from_utf8_unchecked(buffer) })
}

pub async fn write_string<S: AsRef<str>>(stream: &mut tokio::net::UnixStream, string: S) -> std::io::Result<()> {
    let bytes = string.as_ref().as_bytes();

    stream.write_u64(bytes.len() as u64).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await
}