pub fn daemon<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new(format!("{} - daemon", env!("CARGO_PKG_NAME")))
        .arg(
            clap::Arg::with_name("path")
                .index(1)
                .takes_value(true)
                .value_name("mount path")
                .required(true)
                .help("Mount filesystem at this path")
        )
}