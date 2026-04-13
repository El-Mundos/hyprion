use crate::ipc::Message;
use tokio::io::AsyncWriteExt;
use tokio::net::unix::OwnedWriteHalf;

pub struct Client {
    pub id: u64,
    pub domain: Option<String>,     // set if this client is a module
    pub subscriptions: Vec<String>, // event patterns they subscribed to
    writer: OwnedWriteHalf,
}

impl Client {
    pub fn new(id: u64, writer: OwnedWriteHalf) -> Self {
        Self {
            id,
            domain: None,
            subscriptions: Vec::new(),
            writer,
        }
    }

    pub fn matches_event(&self, event_name: &str) -> bool {
        self.subscriptions.iter().any(|pattern| {
            if pattern == "*" {
                return true;
            }
            if let Some(prefix) = pattern.strip_suffix(".*") {
                return event_name.starts_with(&format!("{}.", prefix));
            }
            pattern == event_name
        })
    }

    pub async fn send(&mut self, message: &Message) {
        let Ok(mut json) = serde_json::to_string(message) else {
            eprintln!("Failed to serialize message");
            return;
        };
        json.push('\n');
        if let Err(e) = self.writer.write_all(json.as_bytes()).await {
            eprintln!("Failed to send to client {}: {}", self.id, e);
        }
    }

    pub fn register(&mut self, domain: String) {
        self.domain = Some(domain);
    }

    pub fn subscribe(&mut self, events: Vec<String>) {
        self.subscriptions.extend(events);
    }
}
