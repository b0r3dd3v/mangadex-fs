mod ipc;
mod cli;
use colored::Colorize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = cli::client()
        .version(env!("CARGO_PKG_VERSION"))
        .author(format!("{} <bttrswt@protonmail.com>", env!("CARGO_PKG_AUTHORS")).as_str())
        .about(format!("{}\nThis binary is the client part.", env!("CARGO_PKG_DESCRIPTION")).as_str())
        .get_matches();

    let config = if mangadex_fs::cfg::config_file_path().exists() {
        let maybe_config = mangadex_fs::cfg::Config::load().await?;

        match maybe_config {
            Ok(config) => config,
            Err(error) => {
                println!("{}: invalid configuration file, fallback to defaults: {}", "Warning".yellow(), error);

                mangadex_fs::cfg::Config::default()
            } 
        }
    }
    else { mangadex_fs::cfg::Config::default() };

    let result = match tokio::net::UnixStream::connect(config.socket).await.map(ipc::Client::new) {
        Ok(mut client) => match cli.subcommand() {
            ("kill", _) => client.kill().await,
            ("login", Some(login_args)) => client.log_in(login_args.value_of("username").unwrap(), login_args.value_of("password").unwrap()).await.map(|session| {
                if login_args.is_present("show") {
                    println!("{}: {}\n{}: {}", "session", session.id.cyan(), "rememberme_token", session.remember_me_token.cyan());
                }
            }),
            ("logout", _) => client.log_out().await,
            ("quicksearch", Some(quicksearch_args)) => client.quick_search(quicksearch_args.value_of("query").unwrap()).await.map(|results| {
                if results.len() > 0 {
                    let id_max_len = results.iter().fold(0usize, |acc, result| if acc < result.id.to_string().len() { result.id.to_string().len() } else { acc });
                    let title_max_len = results.iter().fold(0usize, |acc, result| if acc < result.title.len() { result.title.len() } else { acc });

                    for result in results {
                        println!("{:>2$} {:<3$}", result.id.to_string(), result.title.cyan(), id_max_len, title_max_len);
                    }
                }
                else { println!("No results found."); }
            }),
            ("add", Some(add_args)) => match add_args.subcommand() {
                ("manga", Some(manga_args)) => client.add_manga(manga_args.value_of("manga_id").unwrap().parse::<u64>().unwrap()).await.map(|text| {
                    println!("Manga {} has been added.", text.cyan());
                }),
                ("chapter", Some(chapter_args)) => client.add_chapter(chapter_args.value_of("chapter_id").unwrap().parse::<u64>().unwrap()).await.map(|text| {
                    println!("Chapter {} has been added.", text.cyan());
                }),
                (command, _) => Err(ipc::ClientError::Message(format!("unknown subcommand \"add {}\"", command)))
            },
            (command, _) => Err(ipc::ClientError::Message(format!("unknown subcommand \"{}\"", command)))
        },
        Err(error) => Err(ipc::ClientError::IO(error))
    };

    match result {
        Ok(_) => println!("{}", "OK".green()),
        Err(ipc::ClientError::Message(msg)) => println!("{}: {}", "Error".red(), msg),
        Err(ipc::ClientError::IO(error)) => println!("{}: {}", "IO error".red(), error)
    };

    Ok(())
}
