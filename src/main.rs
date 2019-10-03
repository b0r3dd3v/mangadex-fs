#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

mod api;
mod fs;

use clap;
use std::ffi::OsStr;

fn main() {
    env_logger::init();

    let cli = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(format!("{} <bttrswt@protonmail.com>", env!("CARGO_PKG_AUTHORS")).as_str())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            clap::Arg::with_name("path")
                .index(1)
                .takes_value(true)
                .value_name("mount path")
                .required(true)
                .help("Sets the target directory of the mount."),
        )
        .arg(
            clap::Arg::with_name("manga")
                .short("m")
                .long("manga")
                .value_name("manga id")
                .takes_value(true)
                .required(false)
                .multiple(true)
                .help("Mounts corresponding manga by id.")
                .validator(|m| match m.parse::<u64>() {
                    Err(e) => Err(e.to_string()),
                    _ => Ok(()),
                }),
        )
        .arg(
            clap::Arg::with_name("manga-by-title")
                .short("t")
                .long("title")
                .value_name("manga title")
                .takes_value(true)
                .required(false)
                .multiple(true)
                .help("Tries to find manga by title, requires --login.")
                .requires("login")
        )
        .arg(
            clap::Arg::with_name("exact")
                .short("e")
                .long("exact")
                .help("Matches found manga by title. If not set and exact match is not found, defaults to the first match.")
                .takes_value(false)
                .requires("manga-by-title")
        )
        .arg(
            clap::Arg::with_name("mdlist")
                .long("mdlist")
                .short("d")
                .value_name("user id")
                .takes_value(true)
                .required(false)
                .help("Reads a public mdlist to fetch chapters.")
                .validator(|m| match m.parse::<u64>() {
                    Err(e) => Err(e.to_string()),
                    _ => Ok(()),
                }),
        )
        .arg(
            clap::Arg::with_name("threads")
                .short("j")
                .long("threads")
                .value_name("n of threads")
                .takes_value(true)
                .required(false)
                .default_value("1")
                .help("Run FUSE with specified number of threads.")
                .validator(|m| match m.parse::<usize>() {
                    Err(e) => Err(e.to_string()),
                    Ok(n) => {
                        if n == 0 {
                            Err("costanza.jpg".to_string())
                        } else {
                            Ok(())
                        }
                    }
                }),
        )
        .arg(
            clap::Arg::with_name("language")
                .short("l")
                .long("lang")
                .value_name("language code")
                .takes_value(true)
                .required(false)
                .multiple(true)
                .default_value("gb")
                .help("Sets language of fetched chapters, ignores the rest."),
        )
        .arg(
            clap::Arg::with_name("login")
                .long("login")
                .takes_value(false)
                .help("Login with your username and password to enable some additional functionality.")
        )
        .get_matches();

    let client = reqwest::Client::new();

    let session = if cli.is_present("login") {
        use std::io::Write;

        let mut session = Err("Login not successful.");
        let mut attempts = 0;

        while attempts < 3 && session.is_err() {
            let mut login = "".to_owned();

            println!("Logging in");
            print!("Username: ");
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut login).unwrap();

            let password = rpassword::prompt_password_stdout("Password: ").unwrap();

            match api::MangadexSession::login(&client, login, password) {
                Ok(attempt) => session = Ok(attempt),
                Err(e) => {
                    println!("Error: {}", e);
                    attempts = attempts + 1;
                }
            }
        }

        if session.is_err() {
            println!("Too many attempts, exiting...");
            std::process::exit(1);
        } else {
            println!("Logged in successfully!");
        }

        session
    } else {
        Err("Not logged in, try --login flag.".into())
    };

    let mdlist: Option<Vec<u64>> = cli
        .value_of("mdlist")
        .and_then(|id| api::MDList::scrap(&client, id.parse::<u64>().unwrap()).ok())
        .map(|mdlist| mdlist.entries.into_iter().map(|e| e.id).collect());

    let bytitle: Option<Vec<u64>> = cli
        .values_of("manga-by-title")
        .and_then(|titles| {
            titles
                .into_iter()
                .map(|title| {
                    api::MangaByTitle::scrap(
                        &client,
                        &session.as_ref().unwrap(),
                        title,
                        cli.is_present("exact"),
                    )
                    .ok()
                })
                .collect()
        })
        .map(|found: Vec<api::MangaByTitle>| found.into_iter().map(|m| m.0).collect());

    let mut mangadex = fs::MangaDexFS::new(client);

    if let Some(langs) = cli.values_of("language") {
        for lang in langs {
            #[allow(unused_must_use)]
            {
                mangadex.add_language(lang);
            }
        }
    }

    if let Some(manga) = cli.values_of("manga") {
        for id in manga {
            #[allow(unused_must_use)]
            {
                mangadex.add_manga(id.parse::<u64>().unwrap());
            }
        }
    }

    if let Some(mdlist) = mdlist {
        for id in mdlist {
            #[allow(unused_must_use)]
            {
                mangadex.add_manga(id);
            }
        }
    }

    if let Some(bytitle) = bytitle {
        for id in bytitle {
            #[allow(unused_must_use)]
            {
                mangadex.add_manga(id);
            }
        }
    }

    let mountpoint = cli.value_of("path").unwrap();
    let fuse_args: Vec<&OsStr> = vec![&OsStr::new("-oallow_other"), &OsStr::new("-oauto_unmount")];
    let threads = cli.value_of("threads").unwrap().parse::<usize>().unwrap();

    fuse_mt::mount(
        fuse_mt::FuseMT::new(mangadex, threads),
        &mountpoint,
        &fuse_args,
    )
    .unwrap();
}
