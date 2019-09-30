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
                .short("p")
                .long("path")
                .takes_value(true)
                .required(true)
                .help("Sets the target directory of the mount"),
        )
        .arg(
            clap::Arg::with_name("manga")
                .short("m")
                .long("manga")
                .value_name("manga id")
                .takes_value(true)
                .required(false)
                .multiple(true)
                .help("Mounts corresponding manga")
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
                .help("Run FUSE with specified number of threads")
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
                .help("Sets language of fetched chapters, ignores the rest"),
        )
        .get_matches();

    let mut mangadex = fs::MangaDexFS::new();

    for lang in cli.values_of("language").unwrap() {
        mangadex.add_langauge(lang.to_string());
    }

    for id in cli.values_of("manga").unwrap() {
        mangadex.add_manga(id.parse::<u64>().unwrap());
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
