use serde_json::Value;
use std::collections::HashMap;

pub struct CoreState {
    // Domain name → last reported state
    pub cache: HashMap<String, Value>,
    // Domain name → list of commands it supports
    pub registry: HashMap<String, Vec<String>>,
}

impl CoreState {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            registry: HashMap::new(),
        }
    }

    pub fn update_cache(&mut self, domain: String, payload: Value) {
        self.cache.insert(domain, payload);
    }

    pub fn get_cache(&self, domain: &str) -> Option<&Value> {
        self.cache.get(domain)
    }

    pub fn register_module(&mut self, domain: String, commands: Vec<String>) {
        self.registry.insert(domain, commands);
    }

    pub fn is_registered(&self, domain: &str) -> bool {
        self.registry.contains_key(domain)
    }

    pub fn command_exists(&self, domain: &str, action: &str) -> bool {
        self.registry
            .get(domain)
            .map(|commands| commands.iter().any(|c| c == action))
            .unwrap_or(false)
    }
}
