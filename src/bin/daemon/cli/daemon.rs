pub fn daemon<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new(format!("{} - daemon", env!("CARGO_PKG_NAME")))
        .arg(
            clap::Arg::with_name("mountpoint")
                .takes_value(true)
                .value_name("mount point")
                .help("Mount filesystem at this path")
        )
}