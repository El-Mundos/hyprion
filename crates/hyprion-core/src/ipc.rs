use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Message {
    Register {
        domain: String,
        commands: Vec<String>,
    },
    State {
        domain: String,
        payload: Value,
    },
    Command {
        domain: String,
        action: String,
        payload: Option<Value>,
    },
    Query {
        domain: String,
    },
    Subscribe {
        events: Vec<String>,
    },
    Event {
        name: String,
        payload: Value,
    },
}
