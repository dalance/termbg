// This code is based on https://stackoverflow.com/a/75686099

use futures::ready;
use std::{
    fs::File,
    io::{self, Read},
    os::fd::{FromRawFd, RawFd},
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{unix::AsyncFd, AsyncRead, ReadBuf};

// Copied without modification from https://github.com/anowell/nonblock-rs/blob/7685f3060ce9b5dc242847706b541ae46f27340b/src/lib.rs#L179
fn set_blocking(fd: RawFd, blocking: bool) -> io::Result<()> {
    use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};
    let flags = unsafe { fcntl(fd, F_GETFL, 0) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }

    let flags = if blocking {
        flags & !O_NONBLOCK
    } else {
        flags | O_NONBLOCK
    };
    let res = unsafe { fcntl(fd, F_SETFL, flags) };

    if res != 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

pub struct Stdin {
    inner: Option<AsyncFd<File>>,
}

impl Drop for Stdin {
    fn drop(&mut self) {
        let x = self.inner.take().unwrap();
        std::mem::forget(x.into_inner());
        let _ = set_blocking(0, true);
    }
}

// Copied without modification from https://docs.rs/tokio/1.26.0/tokio/io/unix/struct.AsyncFd.html#examples
impl AsyncRead for Stdin {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            let mut guard = ready!(self.inner.as_ref().unwrap().poll_read_ready(cx))?;

            let unfilled = buf.initialize_unfilled();
            match guard.try_io(|inner| inner.get_ref().read(unfilled)) {
                Ok(Ok(len)) => {
                    buf.advance(len);
                    return Poll::Ready(Ok(()));
                }

                Ok(Err(err)) => return Poll::Ready(Err(err)),
                Err(_would_block) => continue,
            }
        }
    }
}

pub fn stdin() -> Result<Stdin, std::io::Error> {
    let stdin_fd = unsafe { File::from_raw_fd(0) };
    set_blocking(0, false)?;
    Ok(Stdin {
        inner: Some(AsyncFd::new(stdin_fd)?),
    })
}
