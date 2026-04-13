mod client;
mod ipc;
mod state;

use std::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

const SOCKET_PATH: &str = "/tmp/hyprion-core.sock";

#[tokio::main]
async fn main() {
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
                match serde_json::from_str::<ipc::Message>(&line) {
                    Ok(message) => {
                        println!("Received: {:?}", message);
                        // TODO: handle message properly once state is implemented
                    }
                    Err(e) => {
                        let error = ipc::Message::Event {
                            name: "error".to_string(),
                            payload: serde_json::json!({"message": e.to_string()}),
                        };
                        let mut json = serde_json::to_string(&error).unwrap();
                        json.push('\n');
                        writer.write_all(json.as_bytes()).await.unwrap();
                    }
                }
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }
}
