# mangadex-fs [![crates.io](https://img.shields.io/crates/v/mangadex-fs?style=flat-square)](https://crates.io/crates/mangadex-fs)

_FUSE driver for your weeb needs._ 

This is a basic implementation of FUSE driver utilising [MangaDex](https://mangadex.org/) undocumented API. Manga information, chapters and individual pages are cached in memory so repeated requests don't accidently DDOS the server.

---

### Version 0.2.0-beta

If you're looking for some _previously_ working code, check out [previous version](https://github.com/bittersweetshimmer/mangadex-fs/tree/v0.1.5). I think it no longer works though.

---

## Usage

`mangadex-fs` consists of 2 binaries, the client part `mangadex-fsc` and the daemon part `mangadex-fsd`.

1. `cargo install mangadex-fs` (or clone this repo),
2. checkout `mangadex-fsc --help` / `mangadex-fsd --help` (or `cargo run --release --bin mangadex-fsc -- --help`) on how to use,
3. please **don't abuse** MangaDex server, use with consideration,
4. have fun reading.  

Short example:
```console
urmom@gay ~> mkdir ~/Manga
urmom@gay ~> mangadex-fsd ~/Manga
```
```console
urmom@gay ~> mangadex-fsc login -u <username> -p <password>
OK
urmom@gay ~> mangadex-fsc search --author "Dowman Sayman" --include anthology supernatural
 4261 Nickelodeon │ Dowman Sayman │ Not followed    │ 2 mo ago
20563 Melancholia │ Dowman Sayman │ Not followed    │ 2 mo ago
OK
urmom@gay ~> mangadex-fsc manga add 20563
Manga Melancholia has been added.
OK
```

### Notes

-   You need to have FUSE installed, and its kernel module loaded: `modprobe fuse`.
-   This version doesn't support any sort of API throttling/debouncing. For now.
-   Since fetching only the chapter page image size no longer works (`curl -I image_url` returns `405`), every time your system issues a `readdir` call (basically `ls`) on a chapter directory, every image gets fetched in its entirety. You can imagine it can take some time. Also MangaDex servers get buried in requests.

    So if you're calling `tree` on the mountpoint directory, you are basically asking for an IP ban.
    
    The `readdir` can also happen if you're using some fancy command line shells (`fish` for example), even if you are not in the chapter directory.
-   API responses are cached, and there is no command for fetching updates currently.
-   ```sh
    cd <mountpoint>/<manga>/<chapter>
    feh --image-bg "black" -Z -. -d -S filename --version-sort
    ```

    creates a good reader. Obviously you need to have [`feh`](https://github.com/derf/feh) installed.
-   You can enable logging by setting `RUST_LOG` environment variable. More [here](https://docs.rs/env_logger/0.7.0/env_logger/).
-   If you encounter a `socket error: Address already in use (os error 98)`, it means the socket file is still present in the runtime directory, you can remove it with `rm $XDG_RUNTIME_DIR/mangadex-fs/mangadex-fsd.sock`.
-   You can place a configuration file in `$XDG_CONFIG_HOME/mangadex-fs/config.toml`. It can be only provided with the socket file path and mountpoint for now, so it's mostly useless:
```toml
mountpoint = "/home/urmom/Manga/"
socket = "/run/user/1000/mangadex-fs/mangadex-fsd.sock"
```

-   >Your code is a dumpster fire

    I bet! This is my first time using Rust for something more complicated than _Hello world_. If you have any guidelines or want to contribute go ahead, any help would be appreciated. With all these mutexes flying around I have no idea what I'm doing.