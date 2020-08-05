#[macro_use]
extern crate log;

mod ipc;
use tokio::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("consider supporting MangaDex at https://mangadex.org/support");

    let config = if mangadex_fs::cfg::config_file_path().exists() {
        let maybe_config = mangadex_fs::cfg::Config::load().await?;

        match maybe_config {
            Ok(config) => config,
            Err(error) => {
                warn!("invalid configuration file, fallback to defaults: {}", error);

                mangadex_fs::cfg::Config::default()
            } 
        }
    }
    else {
        warn!("no configuration file present at \"{}\", fallback to defaults", mangadex_fs::cfg::config_file_path().display());
        mangadex_fs::cfg::Config::default()
    };

    let socket_directory = config.socket.parent().unwrap();

    if !socket_directory.exists() {
        tokio::fs::create_dir_all(socket_directory).await?;
    }

    match tokio::net::UnixListener::bind(&config.socket) {
        Ok(mut listener) => {
            info!("hello");

            let api = std::sync::Arc::new(tokio::sync::Mutex::new(mangadex_fs::api::MangaDexAPI::new()));
            let mut handles: Vec<tokio::task::JoinHandle<()>> = vec![];
            let (kill_tx, mut kill_rx) = tokio::sync::mpsc::channel(1usize);

            let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
            let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;

            loop {
                let mut kill_tx = kill_tx.clone();

                tokio::select! {
                    _ = kill_rx.recv() => {
                        info!("received a kill subcommand, shutting down...");
                        break;
                    },
                    _ = sigint.recv() => {
                        info!("received an interrupt signal, shutting down...");
                        break;
                    },
                    _ = sigterm.recv() => {
                        info!("received a termination signal, shutting down...");
                        break;
                    },
                    maybe_stream = listener.next() => match maybe_stream {
                        Some(Ok(stream)) => {
                            debug!("client connected");

                            let mut connection = ipc::Connection::new(stream, api.clone());

                            handles.push(tokio::spawn(async move {
                                match connection.read_command().await {
                                    Ok(mangadex_fs::ipc::KILL) => kill_tx.send(()).await.unwrap_or(()),
                                    Ok(mangadex_fs::ipc::LOG_IN) => connection.log_in().await.unwrap_or(()),
                                    Ok(mangadex_fs::ipc::LOG_OUT) => connection.log_out().await.unwrap_or(()),
                                    Ok(mangadex_fs::ipc::ADD_MANGA) => connection.add_manga().await.unwrap_or(()),
                                    Ok(mangadex_fs::ipc::ADD_CHAPTER) => connection.add_chapter().await.unwrap_or(()),
                                    Ok(mangadex_fs::ipc::QUICK_SEARCH) => connection.quick_search().await.unwrap_or(()),
                                    Ok(byte) => warn!("invalid client command \"{}\"", byte),
                                    Err(error) => error!("read command IO error: {}", error)
                                };
                            
                                debug!("client disconnected");
                            }));
                        },
                        Some(Err(error)) => warn!("connection to a stream failed: {}", error),
                        None => error!("ph'nglui mglw'nafh Cthulhu R'lyeh wgah'nagl fhtagn")
                        /* 
                         * According to https://tokio-rs.github.io/tokio/doc/tokio/net/struct.UnixListener.html
                         * it's not possible to receive None here.
                         */
                    }
                }
            }

            tokio::fs::remove_file(config.socket).await?;
            info!("goodbye");
        },
        Err(error) => error!("socket error: {}", error)
    }

    Ok(())
}
