use clap::Parser;
use collector::Normaliser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::{create_event_bus, CanonicalEvent, Entity, EntityKind, EventType};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Duration;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(name = "event-input")]
#[command(about = "Feed security events to the SOC detection engine")]
struct Args {
    /// Input file (JSON or JSONL). If not provided, reads from stdin.
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Watch mode: continuously watch the input file for new events.
    #[arg(short, long)]
    watch: bool,

    /// Send events to Rust API via HTTP.
    #[arg(long, default_value = "http://localhost:8080/events")]
    url: String,

    /// Batch size for HTTP requests.
    #[arg(long, default_value = "1")]
    batch_size: usize,
}

#[derive(Debug, Serialize)]
struct ApiEvent {
    event_type: String,
    entity: HashMap<String, String>,
    metadata: HashMap<String, String>,
}

fn canonical_to_api_event(event: &CanonicalEvent) -> ApiEvent {
    let mut entity = HashMap::new();
    entity.insert("kind".to_string(), format!("{:?}", event.src_entity.kind));
    entity.insert("value".to_string(), event.src_entity.key.clone());

    let mut metadata = HashMap::new();
    metadata.insert("dst_entity".to_string(), event.dst_entity.key.clone());
    metadata.insert("event_type".to_string(), format!("{:?}", event.event_type));
    
    for (k, v) in &event.raw_fields {
        metadata.insert(k.clone(), v.clone());
    }

    ApiEvent {
        event_type: format!("{:?}", event.event_type).to_lowercase(),
        entity,
        metadata,
    }
}

fn process_event_line(line: &str) -> Option<CanonicalEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    #[derive(Debug, Deserialize)]
    struct JsonEvent {
        timestamp: Option<String>,
        #[serde(rename = "event_type")]
        event_type: Option<String>,
        username: Option<String>,
        source_ip: Option<String>,
        source_host: Option<String>,
        destination_host: Option<String>,
        destination_ip: Option<String>,
        destination_port: Option<u16>,
        source_port: Option<u16>,
        status: Option<String>,
        process_name: Option<String>,
        command_line: Option<String>,
        bytes_sent: Option<u64>,
        direction: Option<String>,
    }

    if let Ok(event) = serde_json::from_str::<JsonEvent>(trimmed) {
        let src_entity = if let Some(username) = &event.username {
            Entity::new(EntityKind::User, format!("user:{}", username.to_lowercase()))
        } else if let Some(ip) = &event.source_ip {
            Entity::new(EntityKind::IP, format!("ip:{}", ip))
        } else if let Some(host) = &event.source_host {
            Entity::new(EntityKind::Device, format!("device:{}", host))
        } else {
            Entity::new(EntityKind::User, "user:unknown".to_string())
        };

        let dst_entity = if let Some(host) = &event.destination_host {
            Entity::new(EntityKind::Device, format!("device:{}", host))
        } else if let Some(ip) = &event.destination_ip {
            Entity::new(EntityKind::IP, format!("ip:{}", ip))
        } else {
            Entity::new(EntityKind::Device, "device:unknown".to_string())
        };

        let event_type = match event.event_type.as_deref() {
            Some("login_success") | Some("login_failed") => EventType::Login,
            Some("connect") | Some("port_scan") => EventType::Connect,
            Some("access") => EventType::Access,
            Some("escalate") => EventType::Escalate,
            Some("process_spawn") | Some("spawn") => EventType::Spawn,
            _ => EventType::Login,
        };

        let mut raw_fields = HashMap::new();
        if let Some(ref et) = event.event_type {
            raw_fields.insert("event_type".to_string(), et.clone());
        }
        if let Some(ref status) = event.status {
            raw_fields.insert("status".to_string(), status.clone());
        }
        if let Some(ref ip) = event.source_ip {
            raw_fields.insert("source_ip".to_string(), ip.clone());
        }
        if let Some(bytes) = event.bytes_sent {
            raw_fields.insert("bytes".to_string(), bytes.to_string());
        }
        if let Some(ref dir) = event.direction {
            raw_fields.insert("direction".to_string(), dir.clone());
        }

        return Some(CanonicalEvent::new(src_entity, dst_entity, event_type, raw_fields));
    }

    None
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let args = Args::parse();

    info!("Event Input Tool started");
    info!("Target URL: {}", args.url);

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create HTTP client");

    let url = args.url.clone();

    match &args.input {
        Some(path) => {
            info!("Reading from file: {:?}", path);
            process_file(path, args.watch, &client, &url).await;
        }
        None => {
            info!("Reading from stdin...");
            process_stdin(&client, &url).await;
        }
    }
}

async fn send_event(client: &Client, url: &str, event: &ApiEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = client.post(url).json(event).send().await?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        error!("Failed to send event: {}", response.status());
        Err("HTTP error".into())
    }
}

async fn process_stdin(client: &Client, url: &str) {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines().flatten() {
        if let Some(event) = process_event_line(&line) {
            info!("📤 Sending event: {:?}", event.event_type);
            let api_event = canonical_to_api_event(&event);
            if let Err(e) = send_event(client, url, &api_event).await {
                error!("Failed to send: {}", e);
            }
        }
    }
}

async fn process_file(path: &PathBuf, watch: bool, client: &Client, url: &str) {
    info!("Processing events from: {:?}", path);

    if watch {
        // Process file in watch mode
        loop {
            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                for line in reader.lines().flatten() {
                    if let Some(event) = process_event_line(&line) {
                        info!("📤 Event: {:?}", event.event_type);
                        let api_event = canonical_to_api_event(&event);
                        let _ = send_event(client, url, &api_event).await;
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    } else {
        // One-shot mode
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                if let Some(event) = process_event_line(&line) {
                    info!("📤 Event: {:?}", event.event_type);
                    let api_event = canonical_to_api_event(&event);
                    let _ = send_event(client, url, &api_event).await;
                }
            }
        }
    }
}
