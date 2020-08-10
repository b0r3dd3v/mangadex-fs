pub fn client<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new(format!("{} - client", env!("CARGO_PKG_NAME")))
        .subcommand(clap::SubCommand::with_name("kill")
            .about("Kills the mangadex-fsd daemon"))
        .subcommand(clap::SubCommand::with_name("login")
            .about("Create a MangaDex session")
            .arg(clap::Arg::with_name("username")
                .help("MangaDex username")
                .short("u")
                .long("username")
                .takes_value(true)
                .required(true))
            .arg(clap::Arg::with_name("password")
                .help("MangaDex password")
                .short("p")
                .long("password")
                .takes_value(true)
                .required(true))
            .arg(clap::Arg::with_name("show")
                .help("Shows the session token")
                .short("s")
                .long("show")))
        .subcommand(clap::SubCommand::with_name("logout")
            .about("End current session"))
        .subcommand(clap::SubCommand::with_name("search")
            .about("Search")
            .arg(clap::Arg::with_name("title")
                .takes_value(true)
                .required(false)
                .default_value(""))
            .arg(clap::Arg::with_name("author")
                .takes_value(true)
                .short("a")
                .long("author"))
            .arg(clap::Arg::with_name("artist")
                .takes_value(true)
                .short("t")
                .long("artist"))
            .arg(clap::Arg::with_name("language")
                .takes_value(true)
                .short("l")
                .long("language")
                .possible_values(&[
                    "japanese", "jp",
                    "english", "en", "eng", "gb",
                    "polish", "pol", "pl",
                    "russian", "rus", "ru",
                    "german", "ger", "deu", "de",
                    "french", "fre", "fra",
                    "vietnamese", "vie", "vi",
                    "swedish", "swe", "sv",
                    "chinese", "chi", "zho", "zh",
                    "indonesian", "ind", "id",
                    "korean", "kor", "ko",
                    "spanish", "spa", "es",
                    "thai", "tha", "th",
                    "filipino", "fil",
                    "chinese-traditional"
                ]))
            .arg(clap::Arg::with_name("demographic")
                .takes_value(true)
                .short("d")
                .long("demographic")
                .multiple(true)
                .possible_values(&["shounen", "shoujo", "seinen", "josei"]))
            .arg(clap::Arg::with_name("publication")
                .takes_value(true)
                .short("p")
                .long("publication")
                .multiple(true)
                .possible_values(&["ongoing", "completed", "cancelled", "hiatus"]))
            .arg(clap::Arg::with_name("include")
                .takes_value(true)
                .short("i")
                .long("include")
                .multiple(true)
                .possible_values(&[
                    "4koma",
                    "action",
                    "adventure",
                    "awardwinning",
                    "comedy",
                    "cooking",
                    "doujinshi",
                    "drama",
                    "ecchi",
                    "fantasy",
                    "gyaru",
                    "harem",
                    "historical",
                    "horror",
                    "martialarts",
                    "mecha",
                    "medical",
                    "music",
                    "mystery",
                    "oneshot",
                    "psychological",
                    "romance",
                    "schoollife",
                    "scifi",
                    "shoujoai",
                    "shounenai",
                    "sliceoflife",
                    "smut",
                    "sports",
                    "supernatural",
                    "tragedy",
                    "longstrip",
                    "yaoi",
                    "yuri",
                    "videogames",
                    "isekai",
                    "adaptation",
                    "anthology",
                    "webcomic",
                    "fullcolor",
                    "usercreated",
                    "officialcolored",
                    "fancolored",
                    "gore",
                    "sexualviolence",
                    "crime",
                    "magicalgirls",
                    "philosophical",
                    "superhero",
                    "thriller",
                    "wuxia",
                    "aliens",
                    "animals",
                    "crossdressing",
                    "demons",
                    "delinquents",
                    "genderswap",
                    "ghosts",
                    "monstergirls",
                    "loli",
                    "magic",
                    "military",
                    "monsters",
                    "ninja",
                    "officeworkers",
                    "police",
                    "postapocalyptic",
                    "reincarnation",
                    "reverseharem",
                    "samurai",
                    "shota",
                    "survival",
                    "timetravel",
                    "vampires",
                    "traditionalgames",
                    "virtualreality",
                    "zombies",
                    "incest",
                    "mafia",
                ]))
            .arg(clap::Arg::with_name("exclude")
                .takes_value(true)
                .short("e")
                .long("exclude")
                .multiple(true)
                .possible_values(&[
                    "4koma",
                    "action",
                    "adventure",
                    "awardwinning",
                    "comedy",
                    "cooking",
                    "doujinshi",
                    "drama",
                    "ecchi",
                    "fantasy",
                    "gyaru",
                    "harem",
                    "historical",
                    "horror",
                    "martialarts",
                    "mecha",
                    "medical",
                    "music",
                    "mystery",
                    "oneshot",
                    "psychological",
                    "romance",
                    "schoollife",
                    "scifi",
                    "shoujoai",
                    "shounenai",
                    "sliceoflife",
                    "smut",
                    "sports",
                    "supernatural",
                    "tragedy",
                    "longstrip",
                    "yaoi",
                    "yuri",
                    "videogames",
                    "isekai",
                    "adaptation",
                    "anthology",
                    "webcomic",
                    "fullcolor",
                    "usercreated",
                    "officialcolored",
                    "fancolored",
                    "gore",
                    "sexualviolence",
                    "crime",
                    "magicalgirls",
                    "philosophical",
                    "superhero",
                    "thriller",
                    "wuxia",
                    "aliens",
                    "animals",
                    "crossdressing",
                    "demons",
                    "delinquents",
                    "genderswap",
                    "ghosts",
                    "monstergirls",
                    "loli",
                    "magic",
                    "military",
                    "monsters",
                    "ninja",
                    "officeworkers",
                    "police",
                    "postapocalyptic",
                    "reincarnation",
                    "reverseharem",
                    "samurai",
                    "shota",
                    "survival",
                    "timetravel",
                    "vampires",
                    "traditionalgames",
                    "virtualreality",
                    "zombies",
                    "incest",
                    "mafia",
                ]))
            .arg(clap::Arg::with_name("inclusion_mode")
                .takes_value(true)
                .long("inclusion")
                .possible_values(&["all", "any"]))
            .arg(clap::Arg::with_name("exclusion_mode")
                .takes_value(true)
                .long("exclusion")
                .possible_values(&["all", "any"])))
        .subcommand(clap::SubCommand::with_name("add")
            .about("Adds a resource")
            .subcommand(clap::SubCommand::with_name("manga")
                .about("Adds a manga")
                .arg(clap::Arg::with_name("manga_id")
                    .help("ID of the manga")
                    .takes_value(true)
                    .required(true)
                    .validator(|m| match m.parse::<u64>() {
                        Err(e) => Err(e.to_string()),
                        _ => Ok(())
                    }))
                .arg(clap::Arg::with_name("language")
                    .short("l")
                    .long("lang")
                    .help("Adds only chapters in provided language codes")
                    .takes_value(true)
                    .required(false)
                    .multiple(true)
                    .default_value("gb"))))
        .subcommand(clap::SubCommand::with_name("show")
            .subcommand(clap::SubCommand::with_name("mdlist")
                .about("Shows MDList for a given ID. If logged in, also lists authors and last updates")
                .arg(clap::Arg::with_name("mdlist_id")
                    .help("MDList ID")
                    .takes_value(true)
                    .required(true)
                    .validator(|m| match m.parse::<u64>() {
                        Err(e) => Err(e.to_string()),
                        _ => Ok(())
                    }))))
}