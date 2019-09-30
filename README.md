# mangadex-fs [![crates.io](https://img.shields.io/crates/v/mangadex-fs?style=flat-square)](https://crates.io/crates/mangadex-fs)

_FUSE driver for your weeb needs._ 

This is a basic implementation of FUSE driver utilising [MangaDex](https://mangadex.org/) undocumented API. Manga information, chapters and individual pages are cached in memory so excessive requests don't accidently DDOS the server.

---

## Usage

1. `cargo install mangadex-fs` (or clone this repo),
2. checkout `mangadex-fs --help` (or `cargo run --release -- --help`) on how to use,
3. please **don't abuse** MangaDex server, use with consideration,
4. have fun reading.  

### Notes

-   ```sh
    cd <mount>/<manga>/
    ls -1a | grep -e "^1." | xargs -d '\n' feh --image-bg "black" -Z -. -d -S filename --version-sort
    ```

    creates a good reader from matching regular expression *(all chapters from volume 1 in this case)*. Obviously you need to have [`feh`](https://github.com/derf/feh) installed.

    Be careful not to `grep` any substantial amount of chapters or you may find yourself blacklisted on mangadex.

-   You can enable logging by setting `RUST_LOG` environment variable. More [here](https://docs.rs/env_logger/0.7.0/env_logger/).

-   >Your code is a dumpster fire

    I bet! This is my first time using Rust for something more complicated than _Hello world_. If you have any guidelines or want to contribute go ahead, any help would be appreciated.

---

## TODO

- check some edge cases:
  - read more into file permission,
  - ~~multiple chapters formatted to same directory name~~,
  - ~~generating valid entry names~~,
- limit requests frequency,
- adding manga by name,
- ~~attach some uid & gid to entries,~~
- manage filehandles instead of sloppily parsing paths (but hey it works),
- some IPC would be really neat for managing manga at runtime,
- less cloning, mr. borrow checker show me the way,
- ~~maybe publish a crate.~~