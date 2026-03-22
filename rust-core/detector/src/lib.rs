use baseline::BaselineStore;
use bloomfilter::Bloom;
use shared::{CanonicalEvent, EventType};
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tracing::info;

/// Cache for IOC lookup results with 1-hour TTL
pub struct IocCache {
    store: HashMap<String, (f32, Instant)>,
    ttl: Duration,
}

impl IocCache {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            ttl: Duration::from_secs(3600), // 1 hour
        }
    }

    pub fn get(&self, ip: &str) -> Option<f32> {
        if let Some((score, inserted_at)) = self.store.get(ip) {
            if inserted_at.elapsed() < self.ttl {
                return Some(*score);
            }
        }
        None
    }

    pub fn set(&mut self, ip: String, score: f32) {
        self.store.insert(ip, (score, Instant::now()));
    }
}

pub struct Detector {
    sigma_rules: Vec<SigmaRule>,
    baseline_store: Arc<BaselineStore>,
    ioc_bloom: Arc<RwLock<Bloom<String>>>,
    ioc_cache: Arc<Mutex<IocCache>>,
    abuseipdb_key: String,
    thresholds: DetectionThresholds,
}

#[derive(Clone)]
pub struct DetectionThresholds {
    pub sigma_weight: f32,
    pub anomaly_weight: f32,
    pub ioc_weight: f32,
    pub ioc_score_floor: f32,
    pub graph_min_score: f32,
    pub alert_min_score: f32,
}

impl Default for DetectionThresholds {
    fn default() -> Self {
        Self {
            sigma_weight: 0.5,
            anomaly_weight: 0.3,
            ioc_weight: 0.2,
            ioc_score_floor: 0.8,
            graph_min_score: 0.4,
            alert_min_score: 0.7,
        }
    }
}

impl DetectionThresholds {
    pub fn from_config(config: &HashMap<String, f32>) -> Self {
        Self {
            sigma_weight: config.get("sigma_weight").copied().unwrap_or(0.5),
            anomaly_weight: config.get("anomaly_weight").copied().unwrap_or(0.3),
            ioc_weight: config.get("ioc_weight").copied().unwrap_or(0.2),
            ioc_score_floor: config.get("ioc_score_floor").copied().unwrap_or(0.8),
            graph_min_score: config.get("graph_min_score").copied().unwrap_or(0.4),
            alert_min_score: config.get("alert_min_score").copied().unwrap_or(0.7),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SigmaRule {
    pub id: String,
    pub title: String,
    pub logsource_category: String,
    pub condition: String,
    pub threshold: Option<usize>,
    pub timewindow: Option<u64>,
    pub severity: String,
}

impl SigmaRule {
    pub fn from_yaml(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Self::parse_yaml(&content)
    }

    pub fn parse_yaml(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut id = String::new();
        let mut title = String::new();
        let mut logsource_category = String::new();
        let mut condition = String::new();
        let mut threshold = None;
        let mut timewindow = None;
        let mut severity = "medium".to_string();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("title:") {
                title = line.replace("title:", "").trim().to_string();
                id = title.to_lowercase().replace(' ', "_");
            } else if line.starts_with("logsource:") || line.starts_with("  category:") {
                if let Some(cat) = line
                    .replace("logsource:", "")
                    .replace("category:", "")
                    .trim()
                    .split(':')
                    .last()
                {
                    logsource_category = cat.trim().to_string();
                }
            } else if line.starts_with("detection:") || line.starts_with("    condition:") {
                condition = line
                    .replace("detection:", "")
                    .replace("condition:", "")
                    .trim()
                    .to_string();
            } else if line.starts_with("threshold:") {
                threshold = line.replace("threshold:", "").trim().parse().ok();
            } else if line.starts_with("timewindow:") {
                timewindow = line.replace("timewindow:", "").trim().parse().ok();
            } else if line.starts_with("severity:") {
                severity = line.replace("severity:", "").trim().to_string();
            }
        }

        Ok(Self {
            id,
            title,
            logsource_category,
            condition,
            threshold,
            timewindow,
            severity,
        })
    }

    pub fn evaluate(&self, event: &CanonicalEvent) -> bool {
        let event_type_str = format!("{:?}", event.event_type).to_lowercase();
        let event_category = event
            .raw_fields
            .get("category")
            .cloned()
            .unwrap_or_else(|| event_type_str.clone());

        if !self.logsource_category.is_empty()
            && !event_category.contains(&self.logsource_category)
            && self.logsource_category != event_type_str
        {
            return false;
        }

        if self.condition.is_empty() {
            return true;
        }

        for item in self.condition.split(',').map(|s| s.trim()) {
            if let Some((key, value)) = item.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                if !key.contains('|') {
                    let field_value = event
                        .raw_fields
                        .get(key)
                        .cloned()
                        .unwrap_or_default()
                        .to_lowercase();
                    let value_lower = value.to_lowercase();
                    if !value_lower.contains(&field_value) && !field_value.contains(&value_lower) {
                        return false;
                    }
                }
            }
        }

        true
    }

    pub fn severity_score(&self) -> f32 {
        match self.severity.to_lowercase().as_str() {
            "critical" => 1.0,
            "high" => 0.75,
            "medium" => 0.5,
            "low" => 0.25,
            _ => 0.5,
        }
    }
}

impl Detector {
    pub fn new(baseline_store: Arc<BaselineStore>) -> Self {
        Self {
            sigma_rules: Vec::new(),
            baseline_store,
            ioc_bloom: Arc::new(RwLock::new(Bloom::new(10000, 4))),
            ioc_cache: Arc::new(Mutex::new(IocCache::new())),
            abuseipdb_key: std::env::var("ABUSEIPDB_KEY").unwrap_or_default(),
            thresholds: DetectionThresholds::default(),
        }
    }

    pub fn with_thresholds(mut self, thresholds: DetectionThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    pub fn load_sigma_rules(&mut self, rules_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        let entries = fs::read_dir(rules_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .map_or(false, |ext| ext == "yml" || ext == "yaml")
            {
                match SigmaRule::from_yaml(path.to_str().unwrap_or("")) {
                    Ok(rule) => {
                        info!("Loaded sigma rule: {}", rule.title);
                        self.sigma_rules.push(rule);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load sigma rule {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn load_ioc_list(&self, ioc_file: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(ioc_file)?;
        let mut bloom = self.ioc_bloom.write().unwrap();

        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                let owned_line = line.to_string();
                bloom.set(&owned_line);
            }
        }

        info!("Loaded IOCs into bloom filter");
        Ok(())
    }

    pub fn detect(&self, event: &CanonicalEvent) -> DetectionResult {
        let sigma_score = self.evaluate_sigma_rules(event);
        let anomaly_score = self.evaluate_anomaly(event);
        let ioc_score = self.evaluate_ioc(event);

        let mut score = (sigma_score * self.thresholds.sigma_weight)
            + (anomaly_score * self.thresholds.anomaly_weight)
            + (ioc_score * self.thresholds.ioc_weight);

        if ioc_score > 0.0 {
            score = score.max(self.thresholds.ioc_score_floor);
        }

        let should_insert_graph = score >= self.thresholds.graph_min_score;
        let should_alert = score >= self.thresholds.alert_min_score;

        DetectionResult {
            sigma_score,
            anomaly_score,
            ioc_score,
            final_score: score,
            should_insert_graph,
            should_alert,
            matched_sigma_rule: if sigma_score > 0.0 {
                self.sigma_rules
                    .iter()
                    .find(|r| r.evaluate(event))
                    .map(|r| r.id.clone())
            } else {
                None
            },
        }
    }

    fn evaluate_sigma_rules(&self, event: &CanonicalEvent) -> f32 {
        for rule in &self.sigma_rules {
            if rule.evaluate(event) {
                return rule.severity_score();
            }
        }
        0.0
    }

    fn evaluate_anomaly(&self, event: &CanonicalEvent) -> f32 {
        let src_key = &event.src_entity.key;

        let z_score = self.baseline_store.compute_z_score(src_key, 1.0);

        BaselineStore::normalize_z_score(z_score)
    }

    fn evaluate_ioc(&self, event: &CanonicalEvent) -> f32 {
        let bloom = self.ioc_bloom.read().unwrap();

        let check_keys = [&event.src_entity.key, &event.dst_entity.key];

        for key in check_keys {
            if bloom.check(key) {
                return 1.0;
            }
        }

        0.0
    }

    /// Async IOC check against AbuseIPDB with local caching
    pub async fn check_ioc(&self, ip: &str) -> f32 {
        // Skip private IPs
        if self.is_private_ip(ip) {
            return 0.0;
        }

        // Check cache first
        {
            let cache = self.ioc_cache.lock().unwrap();
            if let Some(cached_score) = cache.get(ip) {
                return cached_score;
            }
        }

        // If no API key configured, return 0 gracefully
        if self.abuseipdb_key.is_empty() {
            return 0.0;
        }

        // Call AbuseIPDB API
        let score = self.call_abuseipdb_api(ip).await;

        // Cache the result
        {
            let mut cache = self.ioc_cache.lock().unwrap();
            cache.set(ip.to_string(), score);
        }

        score
    }

    fn is_private_ip(&self, ip: &str) -> bool {
        ip.starts_with("192.168.")
            || ip.starts_with("10.")
            || ip.starts_with("172.16.")
            || ip.starts_with("127.")
            || ip == "localhost"
    }

    async fn call_abuseipdb_api(&self, ip: &str) -> f32 {
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
        {
            Ok(c) => c,
            Err(_) => return 0.0,
        };

        let url = format!(
            "https://api.abuseipdb.com/api/v2/check?ipAddress={}&maxAgeInDays=30",
            ip
        );

        match client
            .get(&url)
            .header("Key", &self.abuseipdb_key)
            .send()
            .await
        {
            Ok(response) => match response.json::<serde_json::Value>().await {
                Ok(json) => {
                    if let Some(data) = json.get("data") {
                        if let Some(score) = data.get("abuseConfidenceScore").and_then(|v| v.as_f64()) {
                            return (score / 100.0) as f32;
                        }
                    }
                    0.0
                }
                Err(_) => 0.0,
            },
            Err(_) => 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DetectionResult {
    pub sigma_score: f32,
    pub anomaly_score: f32,
    pub ioc_score: f32,
    pub final_score: f32,
    pub should_insert_graph: bool,
    pub should_alert: bool,
    pub matched_sigma_rule: Option<String>,
}

impl Default for DetectionResult {
    fn default() -> Self {
        Self {
            sigma_score: 0.0,
            anomaly_score: 0.0,
            ioc_score: 0.0,
            final_score: 0.0,
            should_insert_graph: false,
            should_alert: false,
            matched_sigma_rule: None,
        }
    }
}
