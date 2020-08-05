#[macro_use]
extern crate log;

mod ipc;
mod cli;
use tokio::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = cli::daemon()
        .version(env!("CARGO_PKG_VERSION"))
        .author(format!("{} <bttrswt@protonmail.com>", env!("CARGO_PKG_AUTHORS")).as_str())
        .about(format!("{}\nThis binary is the daemon/filesystem part.", env!("CARGO_PKG_DESCRIPTION")).as_str())
        .get_matches();
    
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

            let mut handles: Vec<tokio::task::JoinHandle<()>> = vec![];
            let (kill_tx, mut kill_rx) = tokio::sync::mpsc::channel::<()>(1usize);

            let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
            let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;

            let mountpoint = cli.value_of("path").unwrap();
            let fuse_args: Vec<&std::ffi::OsStr> = vec![&std::ffi::OsStr::new("-oallow_other"), &std::ffi::OsStr::new("-oauto_unmount")];
            let threads = 8;

            let context = mangadex_fs::Context::new();
            let mangadex = mangadex_fs::MangaDexFS::new(context.clone());

            let _fuse_handle = unsafe { 
                fuse_mt::spawn_mount(fuse_mt::FuseMT::new(mangadex, threads), &mountpoint, &fuse_args)?
            };

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

                            let mut connection = ipc::Connection::new(stream, context.clone());

                            handles.push(tokio::spawn(async move {
                                match connection.read_command().await {
                                    Ok(mangadex_fs::ipc::KILL) => { kill_tx.send(()).await; },
                                    Ok(mangadex_fs::ipc::LOG_IN) => { connection.log_in().await; },
                                    Ok(mangadex_fs::ipc::LOG_OUT) => { connection.log_out().await; },
                                    Ok(mangadex_fs::ipc::ADD_MANGA) => { connection.add_manga().await; },
                                    Ok(mangadex_fs::ipc::ADD_CHAPTER) => { connection.add_chapter().await; },
                                    Ok(mangadex_fs::ipc::QUICK_SEARCH) => { connection.quick_search().await; },
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
