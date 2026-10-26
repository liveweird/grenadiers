use std::{
    io::{self, Write},
    net::SocketAddr,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
    },
    task::{Context, Poll},
};

use axum::serve::Listener;
use tokio::{
    io::{AsyncRead, AsyncWrite, Interest, ReadBuf},
    net::{TcpListener, TcpSocket, TcpStream},
};

pub(super) const REQUESTED_BUFFER_BYTES: u32 = 1_024;
pub(super) type WriteObservationSet = Arc<[Arc<WriteObservations>]>;

#[derive(Default)]
pub(super) struct WriteObservations {
    armed: AtomicBool,
    pressure: AtomicU8,
    pub(super) bytes: AtomicUsize,
    pub(super) calls: AtomicUsize,
    pub(super) syscall_would_block: AtomicUsize,
    pub(super) pending: AtomicUsize,
    pub(super) dropped: AtomicUsize,
}

impl WriteObservations {
    pub(super) fn arm(&self) {
        self.bytes.store(0, Ordering::SeqCst);
        self.calls.store(0, Ordering::SeqCst);
        self.syscall_would_block.store(0, Ordering::SeqCst);
        self.pending.store(0, Ordering::SeqCst);
        self.pressure.store(0, Ordering::SeqCst);
        self.armed.store(true, Ordering::SeqCst);
    }

    pub(super) fn is_pressured(&self) -> bool {
        self.pressure.load(Ordering::SeqCst) == 2
    }

    pub(super) fn load(value: &AtomicUsize) -> usize {
        value.load(Ordering::SeqCst)
    }
}

pub(super) struct ObservedListener {
    inner: TcpListener,
    accepted: usize,
    observations: WriteObservationSet,
}

impl ObservedListener {
    pub(super) fn constrained(
        connection_count: usize,
    ) -> io::Result<(Self, SocketAddr, u32, WriteObservationSet)> {
        let socket = TcpSocket::new_v4()?;
        socket.set_reuseaddr(true)?;
        socket.set_send_buffer_size(REQUESTED_BUFFER_BYTES)?;
        let effective_listener_send_buffer = socket.send_buffer_size()?;
        socket.bind("127.0.0.1:0".parse().expect("valid loopback address"))?;
        let inner = socket.listen(128)?;
        let address = inner.local_addr()?;
        let observations: WriteObservationSet = (0..connection_count)
            .map(|_| Arc::new(WriteObservations::default()))
            .collect();
        Ok((
            Self {
                inner,
                accepted: 0,
                observations: observations.clone(),
            },
            address,
            effective_listener_send_buffer,
            observations,
        ))
    }
}

impl Listener for ObservedListener {
    type Io = ObservedStream;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        let (stream, address) = self.inner.accept().await.expect("test listener accepts");
        let observation = self
            .observations
            .get(self.accepted)
            .unwrap_or_else(|| panic!("unexpected connection {}", self.accepted + 1))
            .clone();
        self.accepted += 1;
        let standard = stream.into_std().expect("accepted stream converts to std");
        let probe = standard.try_clone().expect("accepted stream clones");
        probe
            .set_nonblocking(true)
            .expect("write clone remains nonblocking");
        let inner = TcpStream::from_std(standard).expect("accepted stream converts to Tokio");
        (
            ObservedStream {
                inner,
                probe,
                observation,
            },
            address,
        )
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.inner.local_addr()
    }
}

pub(super) struct ObservedStream {
    inner: TcpStream,
    probe: std::net::TcpStream,
    observation: Arc<WriteObservations>,
}

impl AsyncRead for ObservedStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(context, buffer)
    }
}

impl AsyncWrite for ObservedStream {
    fn poll_write(
        self: Pin<&mut Self>,
        context: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        loop {
            match this.inner.poll_write_ready(context) {
                Poll::Pending => {
                    if this.observation.armed.load(Ordering::SeqCst) {
                        this.observation.pending.fetch_add(1, Ordering::SeqCst);
                        let _ = this.observation.pressure.compare_exchange(
                            1,
                            2,
                            Ordering::SeqCst,
                            Ordering::SeqCst,
                        );
                    }
                    return Poll::Pending;
                }
                Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
                Poll::Ready(Ok(())) => {}
            }
            let observation = &this.observation;
            let armed = observation.armed.load(Ordering::SeqCst);
            let result = this.inner.try_io(Interest::WRITABLE, || {
                let result = this.probe.write(buffer);
                if armed {
                    match &result {
                        Ok(bytes) => {
                            observation.calls.fetch_add(1, Ordering::SeqCst);
                            observation.bytes.fetch_add(*bytes, Ordering::SeqCst);
                            observation.pressure.store(0, Ordering::SeqCst);
                        }
                        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                            observation
                                .syscall_would_block
                                .fetch_add(1, Ordering::SeqCst);
                            observation.pressure.store(1, Ordering::SeqCst);
                        }
                        Err(_) => {}
                    }
                }
                result
            });
            match result {
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => continue,
                result => return Poll::Ready(result),
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(context)
    }
}

impl Drop for ObservedStream {
    fn drop(&mut self) {
        self.observation.dropped.fetch_add(1, Ordering::SeqCst);
    }
}

pub(super) async fn constrained_client(address: SocketAddr) -> io::Result<(TcpStream, u32)> {
    let socket = TcpSocket::new_v4()?;
    socket.set_recv_buffer_size(REQUESTED_BUFFER_BYTES)?;
    let effective = socket.recv_buffer_size()?;
    Ok((socket.connect(address).await?, effective))
}
