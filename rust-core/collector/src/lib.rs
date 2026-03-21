use shared::{CanonicalEvent, Sender};
use std::path::PathBuf;
use tokio::sync::broadcast;
use tracing::{error, info};

pub struct LogCollector {
    event_sender: Sender,
    watch_paths: Vec<PathBuf>,
}

impl LogCollector {
    pub fn new(event_sender: Sender) -> Self {
        Self {
            event_sender,
            watch_paths: Vec::new(),
        }
    }

    pub fn add_watch_path(&mut self, path: PathBuf) {
        info!("Adding watch path: {:?}", path);
        self.watch_paths.push(path);
    }

    pub fn collect_raw_log(&self, raw_log: &str) -> Result<CanonicalEvent, CollectError> {
        use crate::normaliser::Normaliser;

        let normaliser = Normaliser::new();
        normaliser.parse(raw_log)
    }

    pub fn process_line(&self, line: &str) -> Option<CanonicalEvent> {
        match self.collect_raw_log(line) {
            Ok(event) => {
                if let Err(e) = self.event_sender.send(event.clone()) {
                    error!("Failed to send event: {:?}", e);
                }
                Some(event)
            }
            Err(e) => {
                tracing::debug!("Failed to parse log line: {}", e);
                None
            }
        }
    }
}

pub mod normaliser {
    use chrono::{DateTime, Utc};
    use entity_resolver::{Config, EntityResolver};
    use shared::{CanonicalEvent, Entity, EntityKind, EventType};
    use std::collections::HashMap;
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum CollectError {
        #[error("Parse error: {0}")]
        ParseError(String),
        #[error("Normalisation error: {0}")]
        NormalisationError(String),
    }

    pub struct Normaliser {
        resolver: EntityResolver,
    }

    impl Normaliser {
        pub fn new() -> Self {
            let config = Config {
                aliases: {
                    let mut m = HashMap::new();
                    m.insert("ADMIN".to_string(), "user:admin".to_string());
                    m.insert("DOMAIN\\admin".to_string(), "user:admin".to_string());
                    m.insert("ADMINISTRATOR".to_string(), "user:admin".to_string());
                    m
                },
                cidr_groups: {
                    let mut m = HashMap::new();
                    m.insert("10.0.0.0/8".to_string(), "subnet:internal-a".to_string());
                    m.insert(
                        "192.168.0.0/16".to_string(),
                        "subnet:internal-b".to_string(),
                    );
                    m.insert("172.16.0.0/12".to_string(), "subnet:internal-c".to_string());
                    m
                },
            };

            let resolver = EntityResolver::from_config(&config)
                .unwrap_or_else(|_| EntityResolver::new(HashMap::new(), Vec::new()).unwrap());

            Self { resolver }
        }

        pub fn parse(&self, raw_log: &str) -> Result<CanonicalEvent, CollectError> {
            let fields = self.extract_fields(raw_log)?;

            let src_key = fields
                .get("src")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let dst_key = fields
                .get("dst")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let event_type_str = fields
                .get("type")
                .cloned()
                .unwrap_or_else(|| "login".to_string());

            let event_type = EventType::from_str(&event_type_str).unwrap_or(EventType::Login);

            let src_kind = self.detect_entity_kind(&src_key);
            let dst_kind = self.detect_entity_kind(&dst_key);

            let src_entity = self.resolver.resolve(&src_kind, &src_key);
            let dst_entity = self.resolver.resolve(&dst_kind, &dst_key);

            let mut raw_fields = HashMap::new();
            raw_fields.insert("raw".to_string(), raw_log.to_string());
            for (k, v) in &fields {
                raw_fields.insert(k.clone(), v.clone());
            }

            Ok(CanonicalEvent::new(
                src_entity, dst_entity, event_type, raw_fields,
            ))
        }

        fn extract_fields(&self, raw_log: &str) -> Result<HashMap<String, String>, CollectError> {
            let mut fields = HashMap::new();

            let log_lower = raw_log.to_lowercase();

            if log_lower.contains("login")
                || log_lower.contains("logon")
                || log_lower.contains("auth")
            {
                fields.insert("type".to_string(), "login".to_string());
            } else if log_lower.contains("spawn") || log_lower.contains("exec") {
                fields.insert("type".to_string(), "spawn".to_string());
            } else if log_lower.contains("connect") || log_lower.contains("network") {
                fields.insert("type".to_string(), "connect".to_string());
            } else if log_lower.contains("access") || log_lower.contains("read") {
                fields.insert("type".to_string(), "access".to_string());
            } else if log_lower.contains("escalat") || log_lower.contains("privilege") {
                fields.insert("type".to_string(), "escalate".to_string());
            }

            if log_lower.contains("failed") || log_lower.contains("failure") {
                fields.insert("status".to_string(), "failed".to_string());
            } else if log_lower.contains("success") {
                fields.insert("status".to_string(), "success".to_string());
            }

            for part in raw_log.split_whitespace() {
                if part.contains('@') {
                    fields.insert("src".to_string(), part.to_string());
                } else if part.starts_with("src=") {
                    fields.insert("src".to_string(), part[4..].to_string());
                } else if part.starts_with("dst=") {
                    fields.insert("dst".to_string(), part[4..].to_string());
                } else if part.starts_with("user=") {
                    fields.insert("src".to_string(), part[5..].to_string());
                }
            }

            if !fields.contains_key("src") {
                fields.insert("src".to_string(), "unknown".to_string());
            }
            if !fields.contains_key("dst") {
                fields.insert("dst".to_string(), "unknown".to_string());
            }

            Ok(fields)
        }

        fn detect_entity_kind(&self, key: &str) -> EntityKind {
            if key.contains('@') || key.contains('\\') {
                EntityKind::User
            } else if key.parse::<std::net::IpAddr>().is_ok() {
                EntityKind::IP
            } else if key.starts_with("device:") || key.starts_with("host:") {
                EntityKind::Device
            } else if key.starts_with("process:") {
                EntityKind::Process
            } else if key.starts_with("file:") {
                EntityKind::File
            } else {
                EntityKind::User
            }
        }
    }

    impl Default for Normaliser {
        fn default() -> Self {
            Self::new()
        }
    }
}

pub use normaliser::{CollectError, Normaliser};
