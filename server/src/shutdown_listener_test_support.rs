use std::{
    io,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use axum::serve::Listener;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpListener, TcpStream},
    sync::oneshot,
};

pub(super) struct ReadAcknowledgingListener {
    inner: TcpListener,
    expected_bytes: usize,
    read_tx: Option<oneshot::Sender<usize>>,
    dropped_tx: Option<oneshot::Sender<()>>,
}

impl ReadAcknowledgingListener {
    pub(super) fn new(
        inner: TcpListener,
        expected_bytes: usize,
        read_tx: oneshot::Sender<usize>,
        dropped_tx: oneshot::Sender<()>,
    ) -> Self {
        Self {
            inner,
            expected_bytes,
            read_tx: Some(read_tx),
            dropped_tx: Some(dropped_tx),
        }
    }
}

impl Listener for ReadAcknowledgingListener {
    type Io = ReadAcknowledgingStream;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        let (inner, address) = self.inner.accept().await.unwrap();
        (
            ReadAcknowledgingStream {
                inner,
                expected_bytes: self.expected_bytes,
                consumed_bytes: 0,
                read_tx: self.read_tx.take(),
                dropped_tx: self.dropped_tx.take(),
            },
            address,
        )
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.inner.local_addr()
    }
}

pub(super) struct ReadAcknowledgingStream {
    inner: TcpStream,
    expected_bytes: usize,
    consumed_bytes: usize,
    read_tx: Option<oneshot::Sender<usize>>,
    dropped_tx: Option<oneshot::Sender<()>>,
}

impl AsyncRead for ReadAcknowledgingStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let before = buffer.filled().len();
        let result = Pin::new(&mut self.inner).poll_read(context, buffer);
        if let Poll::Ready(Ok(())) = result {
            self.consumed_bytes += buffer.filled().len() - before;
            if self.consumed_bytes >= self.expected_bytes
                && let Some(read_tx) = self.read_tx.take()
            {
                let _ = read_tx.send(self.consumed_bytes);
            }
        }
        result
    }
}

impl AsyncWrite for ReadAcknowledgingStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(context, buffer)
    }

    fn poll_flush(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(context)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(context)
    }
}

impl Drop for ReadAcknowledgingStream {
    fn drop(&mut self) {
        if let Some(dropped_tx) = self.dropped_tx.take() {
            let _ = dropped_tx.send(());
        }
    }
}
