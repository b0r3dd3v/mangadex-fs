pub mod entry;

pub struct MangaDexFS {
    context: std::sync::Arc<crate::Context>
}

impl MangaDexFS {
    pub fn new(context: std::sync::Arc<crate::Context>) -> MangaDexFS {
        MangaDexFS {
            context
        }
    }

    async fn do_lookup(&self, op: &polyfuse::op::Lookup<'_>) -> std::io::Result<polyfuse::reply::ReplyEntry> {
        let entries = self.context.entries.read().await;

        let make_result = |directory: &entry::Directory| -> std::io::Result<polyfuse::reply::ReplyEntry> {
            match directory.entries().into_iter().find(|direntry| direntry.name() == op.name()) {
                // If child direntry is found, find its ino in entries
                Some(child_direntry) => match entries.get(&child_direntry.nodeid()) {
                    // If child inode is found
                    Some(child_inode) => {
                        let mut reply = polyfuse::reply::ReplyEntry::default();
                        let attr = child_inode.get_attr();

                        match attr {
                            Some(attr) => {
                                reply.ino(attr.ino());
                                reply.attr(attr);
                                reply.ttl_attr(std::time::Duration::from_secs(1u64));
                                reply.ttl_entry(std::time::Duration::from_secs(1u64));

                                Ok(reply)
                            },
                            None => Err(std::io::Error::from_raw_os_error(libc::ENOENT))
                        }
                    },
                    None => Err(std::io::Error::from_raw_os_error(libc::ENOENT))
                },
                None => Err(std::io::Error::from_raw_os_error(libc::ENOENT))
            }
        };

        // Find parent entry from op parent
        match entries.get(&op.parent()) {
            Some(entry::Inode(entry::Entry::Root(directory), _)) => make_result(directory),
            Some(entry::Inode(entry::Entry::Manga(_, directory), _)) => make_result(directory),
            Some(entry::Inode(entry::Entry::Chapter(_, directory), _)) => make_result(directory),
            Some(entry::Inode(entry::Entry::ChapterNotFetched(_), _)) => Err(std::io::Error::from_raw_os_error(libc::EINVAL)),
            Some(entry::Inode(entry::Entry::Page(_), _)) => Err(std::io::Error::from_raw_os_error(libc::ENOTDIR)),
            Some(entry::Inode(entry::Entry::Cover(_), _)) => Err(std::io::Error::from_raw_os_error(libc::ENOTDIR)),
            Some(entry::Inode(entry::Entry::External(_), _)) => Err(std::io::Error::from_raw_os_error(libc::ENOTDIR)),
            None => Err(std::io::Error::from_raw_os_error(libc::ENOENT))
        }
    }

    async fn do_getattr(&self, op: &polyfuse::op::Getattr<'_>) -> std::io::Result<polyfuse::reply::ReplyAttr> {
        match self.context.entries.read().await.get(&op.ino()).and_then(|inode| inode.get_attr()) {
            Some(file_attr) => Ok({                
                let mut reply = polyfuse::reply::ReplyAttr::new(file_attr);
                reply.ttl_attr(std::time::Duration::from_secs(1u64));
                reply
            }),
            None => Err(std::io::Error::from_raw_os_error(libc::ENOENT))
        }
    }

    async fn do_read(&self, op: &polyfuse::op::Read<'_>) -> std::io::Result<Vec<u8>> {
        let read_lock = self.context.entries.read().await;

        match read_lock.get(&op.ino()) {
            Some(entry::Inode(entry::Entry::Page(page_ref), _)) => match page_ref.upgrade() {
                Some(page) => Ok(page.0[op.offset() as usize..std::cmp::min(op.offset() as usize + op.size() as usize, page.0.len())].into()),
                None => Err(std::io::Error::from_raw_os_error(libc::EIO))
            },
            Some(entry::Inode(entry::Entry::Cover(cover_ref), _)) => match cover_ref.upgrade() {
                Some(cover) => Ok(cover.0[op.offset() as usize..std::cmp::min(op.offset() as usize + op.size() as usize, cover.0.len())].into()),
                None => Err(std::io::Error::from_raw_os_error(libc::EIO))
            },
            Some(entry::Inode(entry::Entry::External(bytes), _)) => Ok(bytes[op.offset() as usize..std::cmp::min(op.offset() as usize + op.size() as usize, bytes.len())].into()),
            Some(_) => Err(std::io::Error::from_raw_os_error(libc::EINVAL)),
            None => Err(std::io::Error::from_raw_os_error(libc::ENOENT))
        }
    }

    async fn do_readdir(&self, op: &polyfuse::op::Readdir<'_>) -> std::io::Result<Vec<u8>> {
        let make_reply = |directory: &entry::Directory| -> Vec<u8> {
            let entries = {
                let mut entries = vec![
                    polyfuse::DirEntry::dir(".", op.ino(), 1),
                    polyfuse::DirEntry::dir("..", op.ino(), 2)
                ];

                for entry in directory.entries() {
                    entries.push(entry);
                }

                entries
            };

            let mut entries_reply = vec![];
            let mut total_len = 0usize;
            let offset = op.offset() as usize;
            let size = op.size() as usize;

            for entry in entries.iter().skip(offset as usize) {
                let entry = entry.as_ref();
                
                if total_len + entry.len() > size {
                    break;
                }

                entries_reply.extend_from_slice(entry);
                total_len += entry.len();
            }

            entries_reply
        };

        let read_lock = self.context.entries.read().await;

        match read_lock.get(&op.ino()) {
            Some(entry::Inode(entry::Entry::Root(directory), _)) => Ok(make_reply(directory)),
            Some(entry::Inode(entry::Entry::Manga(_, directory), _)) => Ok(make_reply(directory)),
            Some(entry::Inode(entry::Entry::Chapter(_, directory), _)) => Ok(make_reply(directory)),
            Some(entry::Inode(entry::Entry::ChapterNotFetched(chapter_id_ref), _)) => {
                let chapter_id = *chapter_id_ref;

                drop(chapter_id_ref);
                drop(read_lock);

                debug!("chapter not fetched: {}", chapter_id);

                match self.context.get_or_fetch_chapter(chapter_id).await {
                    Ok(_) => {
                        match self.context.entries.read().await.get(&op.ino()) {
                            Some(entry::Inode(entry::Entry::Chapter(_, directory), _)) => Ok(make_reply(directory)),
                            _ => Err(std::io::Error::from_raw_os_error(libc::ENOENT))
                        }
                    },
                    Err(error) => {
                        debug!("chapter fetching error: {}", error);
                        Err(std::io::Error::from_raw_os_error(libc::EIO))
                    }
                }
            },
            Some(entry::Inode(entry::Entry::Page(_), _)) => Err(std::io::Error::from_raw_os_error(libc::ENOTDIR)),
            Some(entry::Inode(entry::Entry::Cover(_), _)) => Err(std::io::Error::from_raw_os_error(libc::ENOTDIR)),
            Some(entry::Inode(entry::Entry::External(_), _)) => Err(std::io::Error::from_raw_os_error(libc::ENOTDIR)),
            None => Err(std::io::Error::from_raw_os_error(libc::ENOENT))
        }
    }
}



#[polyfuse::async_trait]
impl polyfuse::Filesystem for MangaDexFS {
    async fn call<'a, 'cx, T: ?Sized>(
        &'a self,
        cx: &'a mut polyfuse::Context<'cx, T>,
        op: polyfuse::Operation<'cx>,
    ) -> std::io::Result<()>
    where
        T: polyfuse::io::Reader + polyfuse::io::Writer + Unpin + Send,
    {
        macro_rules! try_reply {
            ($e:expr) => {
                match ($e).await {
                    Ok(reply) => {
                        cx.reply(reply).await
                    }
                    Err(err) => {
                        let errno = err.raw_os_error().unwrap_or(libc::EIO);
                        cx.reply_err(errno).await
                    }
                }
            };
        }

        match op {
            polyfuse::Operation::Lookup(op) => try_reply!(self.do_lookup(&op)),
            polyfuse::Operation::Getattr(op) => try_reply!(self.do_getattr(&op)),
            polyfuse::Operation::Read(op) => try_reply!(self.do_read(&op)),
            polyfuse::Operation::Readdir(op) => try_reply!(self.do_readdir(&op)),
            _ => Ok(()),
        }
    }
}