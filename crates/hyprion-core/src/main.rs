mod ipc;

use std::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

const SOCKET_PATH: &str = "/tmp/hyprion-core.sock";

#[tokio::main]
async fn main() {
    // Remove leftover socket from previous run
    let _ = fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH).expect("Failed to bind to socket");

    println!("hyprion-core listening on {}", SOCKET_PATH);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                println!("New connection!");
                tokio::spawn(async move {
                    handle_connection(stream).await;
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_connection(stream: tokio::net::UnixStream) {
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();

        match reader.read_line(&mut line).await {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(_) => {
                let response = match serde_json::from_str::<ipc::Request>(&line) {
                    Ok(request) => handle_request(request),
                    Err(e) => ipc::Response::Error {
                        message: format!("Invalid request: {}", e),
                    },
                };

                let mut json = serde_json::to_string(&response).unwrap();
                json.push('\n');
                writer.write_all(json.as_bytes()).await.unwrap();
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }
}

fn handle_request(request: ipc::Request) -> ipc::Response {
    match request {
        ipc::Request::GetTheme => ipc::Response::Ok,
        ipc::Request::GetVolume => ipc::Response::Volume { level: 0 },
        ipc::Request::SetVolume { level } => ipc::Response::Ok,
    }
}
