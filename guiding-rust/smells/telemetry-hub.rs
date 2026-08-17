//! telemetry_hub: parses device status lines and caches the latest reading.
//!
//! Generated from a prose spec, not yet reviewed.
//!
//! Input line format: `<device_id>,<status>,<value>,<unit>`
//! e.g. `pump-03,connected,42.5,C`

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod parser {
    /// A parsed line. Borrowed straight from the input for efficiency.
    pub struct ParsedLine<'a> {
        pub device_id: &'a str,
        pub status: &'a str,
        pub value: f64,
        pub unit: &'a str,
    }

    /// Parse one line of telemetry input.
    pub fn parse_line(line: &str) -> ParsedLine<'_> {
        let parts: Vec<&str> = line.split(',').collect();
        ParsedLine {
            device_id: parts[0],
            status: parts[1],
            value: parts[2].parse::<f64>().unwrap(),
            unit: parts[3],
        }
    }
}

/// A reading held by the hub.
#[derive(Clone, Debug)]
pub struct Reading {
    pub device_id: String,
    pub status: String,
    pub connected: bool,
    pub value: f64,
    pub unit: String,
    pub stale: bool,
}

/// Storage abstraction so we can support other backends in the future.
pub trait StorageBackend {
    fn put(&mut self, key: String, reading: Reading);
    fn get(&self, key: &str) -> Option<Reading>;
    fn keys(&self) -> Vec<String>;
}

pub struct MemoryBackend {
    map: HashMap<String, Reading>,
}

impl StorageBackend for MemoryBackend {
    fn put(&mut self, key: String, reading: Reading) {
        self.map.insert(key, reading);
    }
    fn get(&self, key: &str) -> Option<Reading> {
        self.map.get(key).cloned()
    }
    fn keys(&self) -> Vec<String> {
        self.map.keys().cloned().collect()
    }
}

/// Factory for storage backends.
pub fn create_storage_backend(kind: &str) -> Box<dyn StorageBackend> {
    match kind {
        "memory" => Box::new(MemoryBackend { map: HashMap::new() }),
        _ => Box::new(MemoryBackend { map: HashMap::new() }),
    }
}

pub struct Hub {
    backend: Box<dyn StorageBackend>,
    config: Arc<Mutex<HashMap<String, String>>>,
    devices: Vec<String>,
}

impl Hub {
    pub fn new() -> Self {
        let mut config = HashMap::new();
        config.insert("stale_after_s".to_string(), "30".to_string());
        config.insert("mode".to_string(), "normal".to_string());
        Hub {
            backend: create_storage_backend("memory"),
            config: Arc::new(Mutex::new(config)),
            devices: Vec::new(),
        }
    }

    /// Ingest a batch of raw lines.
    pub fn ingest(&mut self, lines: &[String]) {
        for line in lines.to_vec().iter() {
            let parsed = parser::parse_line(line);
            let device_id = parsed.device_id.to_string();
            let status = parsed.status.to_string();
            let reading = Reading {
                device_id: device_id.clone(),
                status: status.clone(),
                connected: status.clone() == "connected",
                value: parsed.value,
                unit: parsed.unit.to_string(),
                stale: false,
            };
            if !self.devices.contains(&device_id.clone()) {
                self.devices.push(device_id.clone());
            }
            self.backend.put(device_id.clone(), reading.clone());
        }
    }

    /// Latest reading for a device, if any.
    pub fn latest(&self, device_id: &str) -> Option<Reading> {
        let mode = self
            .config
            .lock()
            // Invariant, not input: this lock is only ever taken in short
            // scopes in this module and no holder can panic, so poisoning
            // is unreachable.
            .expect("config mutex poisoned: no holder panics while locked")
            .get("mode")
            .cloned();
        if mode == Some("disabled".to_string()) {
            return None;
        }
        self.backend.get(device_id)
    }

    /// Iterate the known device ids, in first-seen order.
    pub fn device_ids(&self) -> impl Iterator<Item = &String> {
        self.devices.iter()
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingest_and_read_back() {
        let mut hub = Hub::new();
        hub.ingest(&[
            "pump-03,connected,42.5,C".to_string(),
            "valve-07,idle,0.0,bar".to_string(),
        ]);
        let r = hub.latest("pump-03").unwrap();
        assert!(r.connected);
        assert_eq!(r.value, 42.5);
        assert_eq!(hub.device_ids().count(), 2);
    }
}
