const SORT_PARAMETERS: &[&str] = &["title", "lastupdated", "comments", "views", "follows", "rating"]; 
const DEMOGRAPHIC: &[&str] = &["shounen", "shoujo", "seinen", "josei"];
const PUBLICATION_STATUS: &[&str] = &["ongoing", "completed", "cancelled", "hiatus"];
const LANGUAGES: &[&str] = &[
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
];
const TAGS: &[&str] = &[
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
];
const TAG_MODE: &[&str] = &["all", "any"];
const MDLIST_STATUS: &[&str] = &["reading", "completed", "onhold", "dropped", "plantoread", "rereading"];

fn id_validator(string: String) -> Result<(), String> {
    match string.parse::<u64>() {
        Err(e) => Err(e.to_string()),
        _ => Ok(())
    }
}

pub fn kill<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("kill")
        .about("Kills the mangadex-fsd daemon")
}

pub fn login<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("login")
        .about("Creates a MangaDex session")
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
            .long("show"))
}

pub fn logout<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("logout")
        .about("Ends current session")
}

pub fn search<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("search")
        .about("Search")
        .arg(clap::Arg::with_name("sort")
            .takes_value(true)
            .required(false)
            .short("s")
            .long("sort")
            .conflicts_with("sort_descending")
            .possible_values(SORT_PARAMETERS))
        .arg(clap::Arg::with_name("sort_descending")
            .takes_value(true)
            .required(false)
            .long("sort-descending")
            .conflicts_with("sort")
            .possible_values(SORT_PARAMETERS))
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
            .possible_values(LANGUAGES))
        .arg(clap::Arg::with_name("demographic")
            .takes_value(true)
            .short("d")
            .long("demographic")
            .multiple(true)
            .possible_values(DEMOGRAPHIC))
        .arg(clap::Arg::with_name("publication")
            .takes_value(true)
            .short("p")
            .long("publication")
            .multiple(true)
            .possible_values(PUBLICATION_STATUS))
        .arg(clap::Arg::with_name("include")
            .takes_value(true)
            .short("i")
            .long("include")
            .multiple(true)
            .possible_values(TAGS))
        .arg(clap::Arg::with_name("exclude")
            .takes_value(true)
            .short("e")
            .long("exclude")
            .multiple(true)
            .possible_values(TAGS))
        .arg(clap::Arg::with_name("inclusion_mode")
            .takes_value(true)
            .long("inclusion")
            .possible_values(TAG_MODE))
        .arg(clap::Arg::with_name("exclusion_mode")
            .takes_value(true)
            .long("exclusion")
            .possible_values(TAG_MODE))
}

pub fn chapter_mark<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("mark")
        .about("Marks a chapter as either read on unread")
        .arg(clap::Arg::with_name("chapter_id")
            .help("ID of the chapter")
            .takes_value(true)
            .required(true)
            .validator(id_validator))
        .arg(clap::Arg::with_name("status")
            .takes_value(true)
            .required(true)
            .possible_values(&["read", "unread"]))
}

pub fn chapter<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("chapter")
        .subcommand(chapter_mark())
}

pub fn follows<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("follows")
        .about("Returns the latest updates of followed manga")
}

pub fn manga_add<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("add")
        .about("Adds a manga")
        .arg(clap::Arg::with_name("manga_id")
            .help("ID of the manga")
            .takes_value(true)
            .required(true)
            .validator(id_validator))
        .arg(clap::Arg::with_name("language")
            .short("l")
            .long("lang")
            .help("Adds only chapters in provided language codes")
            .takes_value(true)
            .required(false)
            .multiple(true)
            .default_value("gb"))
}

pub fn manga_follow<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("follow")
        .about("Changes your MDList status of a given manga. Equivalent to `mdlist add`")
        .arg(clap::Arg::with_name("manga_id")
            .help("ID of the manga")
            .takes_value(true)
            .required(true)
            .validator(id_validator))
        .arg(clap::Arg::with_name("status")
            .short("s")
            .long("status")
            .help("Follow with the given status")
            .takes_value(true)
            .required(false)
            .default_value("reading")
            .possible_values(MDLIST_STATUS))
}

pub fn manga_unfollow<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("unfollow")
        .about("Unfollows given manga. Removes 'chapters read' markers. Equivalent to `mdlist remove`")
        .arg(clap::Arg::with_name("manga_id")
            .help("ID of the manga")
            .takes_value(true)
            .required(true)
            .validator(id_validator))
}

pub fn manga<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("manga")
        .subcommand(manga_add())
        .subcommand(manga_follow())
        .subcommand(manga_unfollow())
}

pub fn mdlist_show<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("show")
        .about("Shows MDList for a given ID")
        .arg(clap::Arg::with_name("status")
            .takes_value(true)
            .required(false)
            .short("t")
            .long("status")
            .possible_values(MDLIST_STATUS))
        .arg(clap::Arg::with_name("sort")
            .takes_value(true)
            .required(false)
            .short("s")
            .long("sort")
            .conflicts_with("sort_descending")
            .possible_values(SORT_PARAMETERS))
        .arg(clap::Arg::with_name("sort_descending")
            .takes_value(true)
            .required(false)
            .long("sort-descending")
            .conflicts_with("sort")
            .possible_values(SORT_PARAMETERS))
        .arg(clap::Arg::with_name("mdlist_id")
            .help("MDList ID")
            .takes_value(true)
            .required(true)
            .validator(id_validator))
}

pub fn mdlist_add<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("add")
        .about("Changes your MDList status of a given manga. Equivalent to `manga follow`")
        .arg(clap::Arg::with_name("manga_id")
            .help("ID of the manga")
            .takes_value(true)
            .required(true)
            .validator(id_validator))
        .arg(clap::Arg::with_name("status")
            .short("s")
            .long("status")
            .help("Follow with the given status")
            .takes_value(true)
            .required(false)
            .default_value("reading")
            .possible_values(MDLIST_STATUS))
}

pub fn mdlist_remove<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("remove")
        .about("Unfollows given manga. Removes 'chapters read' markers. Equivalent to `manga unfollow`")
        .arg(clap::Arg::with_name("manga_id")
            .help("ID of the manga")
            .takes_value(true)
            .required(true)
            .validator(id_validator))
}

pub fn mdlist<'a, 'b>() -> clap::App<'a, 'b> {
    clap::SubCommand::with_name("mdlist")
        .subcommand(mdlist_show())
        .subcommand(mdlist_add())
        .subcommand(mdlist_remove())
}

pub fn client<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new(format!("{} - client", env!("CARGO_PKG_NAME")))
        .subcommand(kill())
        .subcommand(login())
        .subcommand(logout())
        .subcommand(search()) 
        .subcommand(chapter())
        .subcommand(follows())
        .subcommand(manga())
        .subcommand(mdlist())
}