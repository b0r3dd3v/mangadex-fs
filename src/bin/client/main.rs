mod ipc;
mod cli;
use colored::Colorize;
use std::convert::TryFrom;

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
                println!("{}: invalid configuration file, fallback to defaults: {}", "Warning".bright_yellow(), error);

                mangadex_fs::cfg::Config::default()
            } 
        }
    }
    else { mangadex_fs::cfg::Config::default() };

    let result = match tokio::net::UnixStream::connect(config.socket).await.map(ipc::Client::new) {
        Ok(mut client) => {
            let result = match cli.subcommand() {
                ("kill", _) => client.kill().await,
                ("login", Some(login_args)) => client.log_in(login_args.value_of("username").unwrap(), login_args.value_of("password").unwrap()).await.map(|session| {
                    if login_args.is_present("show") {
                        println!("{}: {}\n{}: {}", "session".white(), session.id, "rememberme_token".white(), session.remember_me_token);
                    }
                }),
                ("logout", _) => client.log_out().await,
                ("search", Some(search_args)) => {
                    let parsed_params: Result<mangadex_fs::api::SearchParams, ipc::ClientError> = (|| {
                        let mut params = mangadex_fs::api::SearchParams::default();

                        params.title = search_args.value_of("title").unwrap().to_owned();
                        params.author = search_args.value_of("author").map(|x| x.to_owned());
                        params.artist = search_args.value_of("artist").map(|x| x.to_owned());
                        params.original_language = match search_args.value_of("language") { 
                            Some(lang_str) => match mangadex_fs::api::Language::try_from(lang_str) {
                                Ok(language) => Some(language),
                                Err(_) => return Err(ipc::ClientError::Message(format!("Failed to parse language argument: \"{}\"", lang_str)))
                            },
                            None => None
                        };
                        params.demographic = {
                            let mut demographic = mangadex_fs::api::Demographic::default();

                            match search_args.values_of("demographic") {
                                Some(values) => {
                                    demographic.shounen = false;
                                    demographic.shoujo = false;
                                    demographic.seinen = false;
                                    demographic.josei = false;

                                    for value in values {
                                        match value {
                                            "shounen" => demographic.shounen = true,
                                            "shoujo" => demographic.shoujo = true,
                                            "seinen" => demographic.seinen = true,
                                            "josei" => demographic.josei = true,
                                            _ => return Err(ipc::ClientError::Message(format!("Failed to parse demographic argument: \"{}\"", value)))
                                        }
                                    }

                                    demographic
                                },
                                None => demographic
                            }
                        };
                        params.publication = {
                            let mut publication = mangadex_fs::api::PublicationStatus::default();

                            match search_args.values_of("publication") {
                                Some(values) => {
                                    publication.ongoing = false;
                                    publication.completed = false;
                                    publication.cancelled = false;
                                    publication.hiatus = false;

                                    for value in values {
                                        match value {
                                            "ongoing" => publication.ongoing = true,
                                            "completed" => publication.completed = true,
                                            "cancelled" => publication.cancelled = true,
                                            "hiatus" => publication.hiatus = true,
                                            _ => return Err(ipc::ClientError::Message(format!("Failed to parse publication status argument: \"{}\"", value)))
                                        }
                                    }

                                    publication
                                },
                                None => publication
                            }
                        };
                        params.include_tag = {
                            let mut include_tag: Vec<mangadex_fs::api::Genre> = Vec::default();

                            match search_args.values_of("include") {
                                Some(values) => {
                                    for value in values {
                                        match mangadex_fs::api::Genre::try_from(value) {
                                            Ok(tag) => include_tag.push(tag),
                                            _ => return Err(ipc::ClientError::Message(format!("Failed to parse include tag argument: \"{}\"", value)))
                                        }
                                    }

                                    include_tag
                                },
                                None => include_tag
                            }
                        };
                        params.exclude_tag = {
                            let mut exclude_tag: Vec<mangadex_fs::api::Genre> = Vec::default();

                            match search_args.values_of("exclude") {
                                Some(values) => {
                                    for value in values {
                                        match mangadex_fs::api::Genre::try_from(value) {
                                            Ok(tag) => exclude_tag.push(tag),
                                            _ => return Err(ipc::ClientError::Message(format!("Failed to parse include tag argument: \"{}\"", value)))
                                        }
                                    }

                                    exclude_tag
                                },
                                None => exclude_tag
                            }
                        };
                        params.inclusion_mode = match search_args.value_of("inclusion_mode") {
                            Some(value) => {
                                match value {
                                    "all" => mangadex_fs::api::TagMode::All,
                                    "any" => mangadex_fs::api::TagMode::Any,
                                    _ => return Err(ipc::ClientError::Message(format!("Failed to parse tag inclusion mode argument: \"{}\"", value)))
                                }
                            },
                            None => mangadex_fs::api::TagMode::All
                        };
                        params.exclusion_mode = match search_args.value_of("exclusion_mode") {
                            Some(value) => {
                                match value {
                                    "all" => mangadex_fs::api::TagMode::All,
                                    "any" => mangadex_fs::api::TagMode::Any,
                                    _ => return Err(ipc::ClientError::Message(format!("Failed to parse tag inclusion mode argument: \"{}\"", value)))
                                }
                            },
                            None => mangadex_fs::api::TagMode::Any
                        };

                        if let Some(sort_param_str) = search_args.value_of("sort") {
                            params.sort_by = mangadex_fs::api::SortBy(
                                mangadex_fs::api::SortMode::Ascending,
                                mangadex_fs::api::SortParameter::try_from(sort_param_str).unwrap()
                            );
                        }
                        else if let Some(sort_param_str) = search_args.value_of("sort_descending") {
                            params.sort_by = mangadex_fs::api::SortBy(
                                mangadex_fs::api::SortMode::Descending,
                                mangadex_fs::api::SortParameter::try_from(sort_param_str).unwrap()
                            );
                        }

                        Ok(params)
                    })();

                    match parsed_params {
                        Ok(params) => client.search(params).await.map(|results| {
                            if results.len() > 0 {
                                let id_max_len = results.iter().fold(0usize, |acc, result| if acc < result.id.to_string().len() { result.id.to_string().len() } else { acc });
                                let title_max_len = results.iter().fold(0usize, |acc, result| if acc < result.title.len() { result.title.len() } else { acc });
                                let author_max_len: usize = results.iter().fold(0usize, |acc, result| if acc < result.author.len() { result.author.len() } else { acc });

                                for result in &results {
                                    let status = match &result.status {
                                        Some(status) => match status {
                                            mangadex_fs::api::MDListStatus::Completed => status.display().bright_blue(),
                                            mangadex_fs::api::MDListStatus::OnHold => status.display().bright_yellow(),
                                            mangadex_fs::api::MDListStatus::PlanToRead => status.display().white(),
                                            mangadex_fs::api::MDListStatus::Dropped => status.display().bright_red(),
                                            mangadex_fs::api::MDListStatus::Reading => status.display().bright_green(),
                                            mangadex_fs::api::MDListStatus::ReReading => status.display().green()
                                        },
                                        None => "Not followed".bright_black()
                                    };

                                    println!(
                                        "{id:>0$} {title:<1$} {3} {author:<2$} {3} {status:<15} {3} {last_update}",
                                        id_max_len, title_max_len, author_max_len, "│".bright_black(),
                                        id = result.id.to_string().white(),
                                        title = result.title,
                                        author = result.author,
                                        status = status,
                                        last_update = result.last_update
                                    );
                                }

                                if results.len() == 100 {
                                    println!("{}: Search returns a maximum of 100 results, some results may have been omitted.", "Warning".yellow());
                                }
                            }
                            else { println!("No results found."); }
                        }),
                        Err(err) => Err(err)
                    }
                },
                ("add", Some(add_args)) => match add_args.subcommand() {
                    ("manga", Some(manga_args)) => client.add_manga(
                        manga_args.value_of("manga_id").unwrap().parse::<u64>().unwrap(),
                        manga_args.values_of("language").unwrap().map(String::from).collect::<Vec<String>>()
                    ).await.map(|text| {
                        println!("Manga {} has been added.", text.green());
                    }),
                    (command, _) => Err(ipc::ClientError::Message(format!("unknown subcommand \"add {}\"", command)))
                },
                ("show", Some(show_args)) => match show_args.subcommand() {
                    ("mdlist", Some(mdlist_args)) => {
                        let params = (|| {
                            let mut params = mangadex_fs::api::MDListParams::default();

                            params.id = mdlist_args.value_of("mdlist_id").unwrap().parse::<u64>().unwrap();
                            
                            if let Some(sort_param_str) = mdlist_args.value_of("sort") {
                                params.sort_by = mangadex_fs::api::SortBy(
                                    mangadex_fs::api::SortMode::Ascending,
                                    mangadex_fs::api::SortParameter::try_from(sort_param_str).unwrap()
                                );
                            }
                            else if let Some(sort_param_str) = mdlist_args.value_of("sort_descending") {
                                params.sort_by = mangadex_fs::api::SortBy(
                                    mangadex_fs::api::SortMode::Descending,
                                    mangadex_fs::api::SortParameter::try_from(sort_param_str).unwrap()
                                );
                            }

                            if let Some(status_param_str) = mdlist_args.value_of("status") {
                                params.status = match status_param_str {
                                    "reading" => Some(mangadex_fs::api::MDListStatus::Reading),
                                    "completed" => Some(mangadex_fs::api::MDListStatus::Completed),
                                    "onhold" => Some(mangadex_fs::api::MDListStatus::OnHold),
                                    "dropped" => Some(mangadex_fs::api::MDListStatus::Dropped),
                                    "plantoread" => Some(mangadex_fs::api::MDListStatus::PlanToRead),
                                    "rereading" => Some(mangadex_fs::api::MDListStatus::ReReading),
                                    _ => return Err(ipc::ClientError::Message(format!("Failed to parse status argument: \"{}\"", status_param_str)))
                                };
                            }

                            Ok(params)
                        })();

                        println!("params: {:?}", params);

                        match params {
                            Ok(params) => client.mdlist(params).await.map(|results| {
                                
                                if results.len() > 0 {
                                    let id_max_len = results.iter().fold(0usize, |acc, result| if acc < result.id.to_string().len() { result.id.to_string().len() } else { acc });
                                    let title_max_len = results.iter().fold(0usize, |acc, result| if acc < result.title.len() { result.title.len() } else { acc });
                                    let author_max_len: usize = results.iter().fold(0usize, |acc, result| if acc < result.author.len() { result.author.len() } else { acc });

                                    for result in &results {
                                        let status = match &result.status {
                                            mangadex_fs::api::MDListStatus::Completed => result.status.display().bright_blue(),
                                            mangadex_fs::api::MDListStatus::OnHold => result.status.display().bright_yellow(),
                                            mangadex_fs::api::MDListStatus::PlanToRead => result.status.display().white(),
                                            mangadex_fs::api::MDListStatus::Dropped => result.status.display().bright_red(),
                                            mangadex_fs::api::MDListStatus::Reading => result.status.display().bright_green(),
                                            mangadex_fs::api::MDListStatus::ReReading => result.status.display().green()
                                        };

                                        println!(
                                            "{id:>0$} {title:<1$} {3} {author:<2$} {3} {status:<15} {3} {last_update}",
                                            id_max_len, title_max_len, author_max_len, "│".bright_black(),
                                            id = result.id.to_string().white(),
                                            title = result.title,
                                            author = result.author,
                                            status = status,
                                            last_update = result.last_update
                                        );
                                    }

                                    if results.len() == 100 {
                                        println!("{}: MDList returns a maximum of 100 entries, some entries may have been omitted.", "Warning".bright_yellow());
                                    }
                                }
                                else {
                                    println!("MDList contains no entries or is private.");
                                }
                            }),
                            Err(err) => Err(err)
                        }
                    },
                    (command, _) => Err(ipc::ClientError::Message(format!("unknown subcommand \"add {}\"", command)))
                },
                (command, _) => Err(ipc::ClientError::Message(format!("unknown subcommand \"{}\"", command)))
            };

            client.end_connection().await.ok();
            result
        },
        Err(error) => Err(ipc::ClientError::IO(error))
    };

    match result {
        Ok(_) => println!("{}", "OK".bright_green()),
        Err(ipc::ClientError::Message(msg)) => println!("{}: {}", "Error".bright_red(), msg),
        Err(ipc::ClientError::IO(error)) => println!("{}: {}", "IO error".bright_red(), error)
    };

    Ok(())
}
