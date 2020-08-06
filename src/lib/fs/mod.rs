pub mod entry;

pub struct MangaDexFS {
    context: std::sync::Arc<crate::Context>,
    uid: nix::unistd::Uid,
    gid: nix::unistd::Gid
}

fn root_attr(uid: nix::unistd::Uid, gid: nix::unistd::Gid) -> polyfuse::FileAttr {
    let mut attr = polyfuse::FileAttr::default();
    attr.set_size(4096u64);
    attr.set_blocks(4u64);
    attr.set_atime(std::time::SystemTime::now());
    attr.set_mtime(std::time::SystemTime::now());
    attr.set_ctime(std::time::SystemTime::now());
    attr.set_mode(libc::S_IFDIR as u32 | 0o555);
    attr.set_nlink(2);
    attr.set_uid(uid.as_raw() as u32);
    attr.set_gid(gid.as_raw() as u32);
    attr.set_rdev(0u32);
    attr
}

impl MangaDexFS {
    pub fn new(uid: nix::unistd::Uid, gid: nix::unistd::Gid, context: std::sync::Arc<crate::Context>) -> MangaDexFS {
        MangaDexFS {
            context, uid, gid
        }
    }

    async fn do_lookup(&self, op: &polyfuse::op::Lookup<'_>) -> std::io::Result<polyfuse::reply::ReplyEntry> {
        if op.parent() == 1 {
            let reply = {
                let mut reply = polyfuse::reply::ReplyEntry::default();
                
                reply.ino(1);
                reply.generation(0u64);
                reply.ttl_entry(std::time::Duration::new(0u64, 0u32));
                reply.ttl_attr(std::time::Duration::new(0u64, 0u32));
                reply.attr(root_attr(self.uid, self.gid));
                reply
            };

            Ok(reply)
        }
        else {
            Err(std::io::Error::from_raw_os_error(libc::EINVAL))
        }
    }

    async fn do_getattr(&self, op: &polyfuse::op::Getattr<'_>) -> std::io::Result<polyfuse::reply::ReplyAttr> {
        debug!("getattr {:?}", op);

        if op.ino() == 1 {
            let reply = {
                let mut reply = polyfuse::reply::ReplyAttr::new(root_attr(self.uid, self.gid));
                reply.ttl_attr(std::time::Duration::from_millis(0u64));
                reply
            };

            Ok(reply)
        }
        else {
            Err(std::io::Error::from_raw_os_error(libc::ENOENT))
        }
    }

    async fn do_read(&self, op: &polyfuse::op::Read<'_>) -> std::io::Result<&[u8]> {
        debug!("read {:?}", op);

        Err(std::io::Error::from_raw_os_error(libc::ENOENT))
    }

    async fn do_readdir(&self, op: &polyfuse::op::Readdir<'_>) -> std::io::Result<Vec<u8>> {
        debug!("readdir {:?}", op);

        if op.ino() != 1 {
            Err(std::io::Error::from_raw_os_error(libc::ENOTDIR))
        }
        else {
            let entries = vec![
                polyfuse::DirEntry::dir(".", 1, 1),
                polyfuse::DirEntry::dir("..", 1, 2)
            ];

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

            Ok(entries_reply)
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