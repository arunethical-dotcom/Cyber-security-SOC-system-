use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityKind {
    User,
    IP,
    Device,
    Process,
    File,
}

impl std::fmt::Display for EntityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityKind::User => write!(f, "User"),
            EntityKind::IP => write!(f, "IP"),
            EntityKind::Device => write!(f, "Device"),
            EntityKind::Process => write!(f, "Process"),
            EntityKind::File => write!(f, "File"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entity {
    pub kind: EntityKind,
    pub key: String,
}

impl Entity {
    pub fn new(kind: EntityKind, key: String) -> Self {
        Self { kind, key }
    }

    pub fn user(key: impl Into<String>) -> Self {
        Self {
            kind: EntityKind::User,
            key: key.into(),
        }
    }

    pub fn ip(key: impl Into<String>) -> Self {
        Self {
            kind: EntityKind::IP,
            key: key.into(),
        }
    }

    pub fn device(key: impl Into<String>) -> Self {
        Self {
            kind: EntityKind::Device,
            key: key.into(),
        }
    }

    pub fn process(key: impl Into<String>) -> Self {
        Self {
            kind: EntityKind::Process,
            key: key.into(),
        }
    }

    pub fn file(key: impl Into<String>) -> Self {
        Self {
            kind: EntityKind::File,
            key: key.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventType {
    Login,
    Spawn,
    Connect,
    Access,
    Escalate,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Login => write!(f, "Login"),
            EventType::Spawn => write!(f, "Spawn"),
            EventType::Connect => write!(f, "Connect"),
            EventType::Access => write!(f, "Access"),
            EventType::Escalate => write!(f, "Escalate"),
        }
    }
}

impl EventType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "login" => Some(EventType::Login),
            "spawn" => Some(EventType::Spawn),
            "connect" => Some(EventType::Connect),
            "access" => Some(EventType::Access),
            "escalate" => Some(EventType::Escalate),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub src_entity: Entity,
    pub dst_entity: Entity,
    pub event_type: EventType,
    pub raw_fields: HashMap<String, String>,
    pub score: f32,
}

impl CanonicalEvent {
    pub fn new(
        src_entity: Entity,
        dst_entity: Entity,
        event_type: EventType,
        raw_fields: HashMap<String, String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            src_entity,
            dst_entity,
            event_type,
            raw_fields,
            score: 0.0,
        }
    }
}

pub mod bus {
    use tokio::sync::broadcast;

    pub fn create_event_bus() -> (Sender, Receiver) {
        let (tx, rx) = broadcast::channel(1024);
        (Sender { tx }, Receiver { rx })
    }

    #[derive(Clone)]
    pub struct Sender {
        tx: broadcast::Sender<crate::CanonicalEvent>,
    }

    impl Sender {
        pub fn send(
            &self,
            event: crate::CanonicalEvent,
        ) -> Result<(), broadcast::error::SendError<crate::CanonicalEvent>> {
            self.tx.send(event).map(|_| ())
        }
    }

    pub struct Receiver {
        rx: broadcast::Receiver<crate::CanonicalEvent>,
    }

    impl Receiver {
        pub async fn recv(&mut self) -> Result<crate::CanonicalEvent, broadcast::error::RecvError> {
            self.rx.recv().await
        }

        pub fn blocking_recv(
            &mut self,
        ) -> Result<crate::CanonicalEvent, broadcast::error::RecvError> {
            self.rx.blocking_recv()
        }
    }
}

pub use bus::{create_event_bus, Receiver, Sender};
