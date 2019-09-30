# MangaDexFS
_FUSE driver for your weeb needs._ 

---
This is a basic implementation of FUSE driver utilising [MangaDex](https://mangadex.org/) undocumented API. Manga information, chapters and individual pages are cached in memory so excessive requests don't accidently DDOS the server.

### Usage
---
1. clone this repo,
2. build `cargo build --release`,
3. checkout `mangadex-fs --help` on how to use,
4. please **don't abuse** MangaDex server, use with consideration,
5. have fun reading.  

### TODO
---
- check some edge cases:
  - read more into file permission,
  - multiple chapters formatted to same directory name,
  - generating valid entry names,
- limit requests frequency,
- adding manga by name,
- ~~attach some uid & gid to entries,~~
- manage filehandles instead of sloppily parsing paths (but hey it works),
- some IPC would be really neat for managing manga at runtime,
- less cloning, mr. borrow checker show me the way,
- maybe publish a crate.

### _>Your code is a dumpster fire_
---
I bet! This is my first time using Rust for something more complicated than _Hello world_. If you have any guidelines or want to contribute go ahead, any help would be appreciated.