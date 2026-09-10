use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::{Sink, Stream};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Bytes, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub struct WsIo {
    inner: WebSocketStream<MaybeTlsStream<TcpStream>>,
    pending: Bytes,
    closed: bool,
}

impl WsIo {
    pub const fn new(inner: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self {
            inner,
            pending: Bytes::new(),
            closed: false,
        }
    }
}

impl AsyncRead for WsIo {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            if !self.pending.is_empty() {
                let n = buf.remaining().min(self.pending.len());
                let chunk = self.pending.split_to(n);
                buf.put_slice(&chunk);
                return Poll::Ready(Ok(()));
            }
            if self.closed {
                return Poll::Ready(Ok(()));
            }
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => {
                    self.closed = true;
                    return Poll::Ready(Ok(()));
                }
                Poll::Ready(Some(Err(err))) => return Poll::Ready(Err(io::Error::other(err))),
                Poll::Ready(Some(Ok(message))) => match message {
                    Message::Binary(bytes) => self.pending = bytes,
                    Message::Text(text) => self.pending = Bytes::from(text),
                    Message::Close(_) => {
                        self.closed = true;
                        return Poll::Ready(Ok(()));
                    }
                    _ => {}
                },
            }
        }
    }
}

impl AsyncWrite for WsIo {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        data: &[u8],
    ) -> Poll<io::Result<usize>> {
        match Pin::new(&mut self.inner).poll_ready(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(err)) => Poll::Ready(Err(io::Error::other(err))),
            Poll::Ready(Ok(())) => {
                Pin::new(&mut self.inner)
                    .start_send(Message::Binary(Bytes::copy_from_slice(data)))
                    .map_err(io::Error::other)?;
                Poll::Ready(Ok(data.len()))
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner)
            .poll_flush(cx)
            .map_err(io::Error::other)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner)
            .poll_close(cx)
            .map_err(io::Error::other)
    }
}
