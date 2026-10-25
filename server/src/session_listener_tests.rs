use std::time::Duration;

use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinHandle,
    time::timeout,
};

use crate::{AppState, app};

const ORIGIN: &str = "http://localhost:5177";

async fn start_server(state: AppState) -> (std::net::SocketAddr, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app(state, None).into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    (address, task)
}

#[tokio::test]
async fn real_listener_connect_info_reaches_session_minting() {
    let (address, server) = start_server(AppState::without_database([ORIGIN.to_owned()])).await;
    let stream = TcpStream::connect(address).await.unwrap();
    let request = format!(
        "POST /api/v1/game/session HTTP/1.1\r\nHost: {address}\r\nOrigin: {ORIGIN}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    let mut remaining = request.as_bytes();
    while !remaining.is_empty() {
        stream.writable().await.unwrap();
        match stream.try_write(remaining) {
            Ok(written) => remaining = &remaining[written..],
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) => panic!("request write failed: {error}"),
        }
    }

    let response = timeout(Duration::from_secs(2), async {
        let mut response = Vec::new();
        loop {
            stream.readable().await.unwrap();
            let mut buffer = [0_u8; 1_024];
            match stream.try_read(&mut buffer) {
                Ok(0) => break response,
                Ok(read) => response.extend_from_slice(&buffer[..read]),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(error) => panic!("response read failed: {error}"),
            }
        }
    })
    .await
    .expect("session response timed out");
    let response = String::from_utf8(response).expect("HTTP response is UTF-8");
    assert!(response.starts_with("HTTP/1.1 201 Created\r\n"));
    assert!(response.contains("cache-control: no-store"));
    assert!(response.contains("\"token\":"));
    server.abort();
}
