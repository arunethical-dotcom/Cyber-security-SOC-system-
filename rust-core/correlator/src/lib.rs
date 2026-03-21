use chrono::{DateTime, Duration, Utc};
use graph::{EventEdge, EventGraph};
use serde::{Deserialize, Serialize};
use shared::{Entity, EventType};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use tracing::info;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub severity: String,
    pub chain: Vec<String>,
    pub entities: Vec<String>,
    pub sigma_score: f32,
    pub z_score: f32,
    pub ioc_match: Option<String>,
    pub cvss: f32,
    pub base_signal: f32,
    pub summary: Option<String>,
    pub actions: Option<Vec<String>>,
}

impl Incident {
    pub fn new(chain_type: &ChainType, entities: Vec<String>, base_signal: f32) -> Self {
        let (severity, chain, cvss) = chain_type.metadata();

        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            severity,
            chain,
            entities,
            sigma_score: base_signal,
            z_score: 0.0,
            ioc_match: None,
            cvss,
            base_signal,
            summary: None,
            actions: None,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum ChainType {
    BruteForceSuccess,
    LateralMove,
    ExfilCandidate,
    PrivilegeEscalate,
}

impl ChainType {
    pub fn metadata(&self) -> (String, Vec<String>, f32) {
        match self {
            ChainType::BruteForceSuccess => (
                "HIGH".to_string(),
                vec!["T1110".to_string(), "TA0001".to_string()],
                7.5,
            ),
            ChainType::LateralMove => (
                "HIGH".to_string(),
                vec!["T1021".to_string(), "TA0008".to_string()],
                8.0,
            ),
            ChainType::ExfilCandidate => (
                "CRITICAL".to_string(),
                vec!["T1041".to_string(), "TA0010".to_string()],
                9.0,
            ),
            ChainType::PrivilegeEscalate => (
                "MEDIUM".to_string(),
                vec!["T1068".to_string(), "TA0004".to_string()],
                6.5,
            ),
        }
    }
}

#[derive(Clone)]
pub struct CorrelationThresholds {
    pub max_chain_window_secs: i64,
    pub lateral_move_window_secs: i64,
    pub priv_esc_window_secs: i64,
    pub brute_force_min_attempts: usize,
    pub exfil_min_zscore: f32,
    pub asset_criticality: HashMap<String, f32>,
}

impl Default for CorrelationThresholds {
    fn default() -> Self {
        Self {
            max_chain_window_secs: 600,
            lateral_move_window_secs: 900,
            priv_esc_window_secs: 300,
            brute_force_min_attempts: 5,
            exfil_min_zscore: 4.0,
            asset_criticality: {
                let mut m = HashMap::new();
                m.insert("subnet:internal-a".to_string(), 2.0);
                m.insert("user:admin".to_string(), 2.0);
                m.insert("device:dc01".to_string(), 2.0);
                m
            },
        }
    }
}

impl CorrelationThresholds {
    pub fn from_config(config: &HashMap<String, serde_json::Value>) -> Self {
        let mut thresholds = Self::default();

        if let Some(v) = config.get("max_chain_window_secs").and_then(|v| v.as_i64()) {
            thresholds.max_chain_window_secs = v;
        }
        if let Some(v) = config
            .get("lateral_move_window_secs")
            .and_then(|v| v.as_i64())
        {
            thresholds.lateral_move_window_secs = v;
        }
        if let Some(v) = config.get("priv_esc_window_secs").and_then(|v| v.as_i64()) {
            thresholds.priv_esc_window_secs = v;
        }
        if let Some(v) = config
            .get("brute_force_min_attempts")
            .and_then(|v| v.as_u64())
        {
            thresholds.brute_force_min_attempts = v as usize;
        }
        if let Some(v) = config.get("exfil_min_zscore").and_then(|v| v.as_f64()) {
            thresholds.exfil_min_zscore = v as f32;
        }

        thresholds
    }

    pub fn get_asset_criticality(&self, entity: &str) -> f32 {
        self.asset_criticality.get(entity).copied().unwrap_or(1.0)
    }
}

pub struct Correlator {
    graph: Arc<RwLock<EventGraph>>,
    thresholds: CorrelationThresholds,
    incident_store: Arc<RwLock<VecDequeWithMax<Incident>>>,
}

struct VecDequeWithMax<T> {
    data: std::collections::VecDeque<T>,
    max_capacity: usize,
}

impl<T> VecDequeWithMax<T> {
    fn new(max_capacity: usize) -> Self {
        Self {
            data: std::collections::VecDeque::new(),
            max_capacity,
        }
    }

    fn push_back(&mut self, item: T) {
        if self.data.len() >= self.max_capacity {
            self.data.pop_front();
        }
        self.data.push_back(item);
    }

    fn get_all(&self) -> Vec<&T> {
        self.data.iter().collect()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut()
    }
}

impl Correlator {
    pub fn new(graph: Arc<RwLock<EventGraph>>) -> Self {
        Self {
            graph,
            thresholds: CorrelationThresholds::default(),
            incident_store: Arc::new(RwLock::new(VecDequeWithMax::new(10000))),
        }
    }

    pub fn with_thresholds(mut self, thresholds: CorrelationThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    pub fn correlate(&self, event_edges: Vec<&EventEdge>) -> Option<Incident> {
        if event_edges.is_empty() {
            return None;
        }

        if let Some(incident) = self.detect_brute_force(&event_edges) {
            return Some(incident);
        }

        if let Some(incident) = self.detect_lateral_move(&event_edges) {
            return Some(incident);
        }

        if let Some(incident) = self.detect_exfil(&event_edges) {
            return Some(incident);
        }

        if let Some(incident) = self.detect_privilege_escalate(&event_edges) {
            return Some(incident);
        }

        None
    }

    fn detect_brute_force(&self, edges: &[&EventEdge]) -> Option<Incident> {
        let mut failed_logins: Vec<&&EventEdge> = edges
            .iter()
            .filter(|e| e.event_type == EventType::Login)
            .filter(|e| {
                e.sigma_rule_id
                    .as_ref()
                    .map_or(true, |id| id.contains("failed") || e.score > 0.5)
            })
            .collect();

        if failed_logins.len() < self.thresholds.brute_force_min_attempts {
            return None;
        }

        failed_logins.sort_by_key(|e| e.timestamp);

        let first_failed = failed_logins.first()?.timestamp;
        let last_failed = failed_logins.last()?.timestamp;

        if (last_failed - first_failed).num_seconds() > self.thresholds.max_chain_window_secs {
            return None;
        }

        let src_key = "src_entity";
        let entities = vec![src_key.to_string()];

        let mut incident = Incident::new(&ChainType::BruteForceSuccess, entities, 0.8);

        let tactic_span_weight = 1.25;
        let asset_crit = self.thresholds.get_asset_criticality(src_key);
        incident.base_signal *= tactic_span_weight * asset_crit;

        info!("Detected brute force attack: {}", incident.id);

        Some(incident)
    }

    fn detect_lateral_move(&self, edges: &[&EventEdge]) -> Option<Incident> {
        let login_edges: Vec<&&EventEdge> = edges
            .iter()
            .filter(|e| e.event_type == EventType::Login)
            .collect();

        if login_edges.is_empty() {
            return None;
        }

        for login in login_edges {
            let login_time = login.timestamp;

            let outbound_edges: Vec<&&EventEdge> = edges
                .iter()
                .filter(|e| {
                    e.event_type == EventType::Connect
                        && (login_time - e.timestamp).num_seconds().abs()
                            <= self.thresholds.lateral_move_window_secs
                })
                .filter(|e| {
                    !e.sigma_rule_id.as_ref().map_or(false, |id| {
                        id.contains("80") || id.contains("443") || id.contains("22")
                    })
                })
                .collect();

            if !outbound_edges.is_empty() {
                let entities: Vec<String> = vec!["source".to_string(), "target".to_string()];

                let mut incident = Incident::new(&ChainType::LateralMove, entities, 0.7);

                let tactic_span_weight = 1.25;
                incident.base_signal *= tactic_span_weight;

                info!("Detected lateral movement: {}", incident.id);

                return Some(incident);
            }
        }

        None
    }

    fn detect_exfil(&self, edges: &[&EventEdge]) -> Option<Incident> {
        let access_edges: Vec<&&EventEdge> = edges
            .iter()
            .filter(|e| e.event_type == EventType::Access)
            .collect();

        if access_edges.is_empty() {
            return None;
        }

        let connect_edges: Vec<&&EventEdge> = edges
            .iter()
            .filter(|e| e.event_type == EventType::Connect)
            .filter(|e| {
                e.sigma_rule_id.as_ref().map_or(false, |id| {
                    id.contains("outbound") || id.contains("external")
                })
            })
            .collect();

        for _ in access_edges {
            for connect in &connect_edges {
                let z_score = connect.score * 6.0;

                if z_score >= self.thresholds.exfil_min_zscore {
                    let entities: Vec<String> =
                        vec!["internal_host".to_string(), "external_ip".to_string()];

                    let mut incident = Incident::new(&ChainType::ExfilCandidate, entities, 0.9);

                    let tactic_span_weight = 1.25;
                    let asset_crit = self.thresholds.get_asset_criticality("internal_host");
                    incident.base_signal *= tactic_span_weight * asset_crit;
                    incident.z_score = z_score;

                    info!("Detected potential exfiltration: {}", incident.id);

                    return Some(incident);
                }
            }
        }

        None
    }

    fn detect_privilege_escalate(&self, edges: &[&EventEdge]) -> Option<Incident> {
        let login_edges: Vec<&&EventEdge> = edges
            .iter()
            .filter(|e| e.event_type == EventType::Login)
            .collect();

        for login in login_edges {
            let login_time = login.timestamp;

            let escalate_edges: Vec<&&EventEdge> = edges
                .iter()
                .filter(|e| e.event_type == EventType::Escalate)
                .filter(|e| {
                    (e.timestamp - login_time).num_seconds().abs()
                        <= self.thresholds.priv_esc_window_secs
                })
                .collect();

            for escalate in &escalate_edges {
                let escalate_time = escalate.timestamp;

                let access_edges: Vec<&&EventEdge> = edges
                    .iter()
                    .filter(|e| e.event_type == EventType::Access)
                    .filter(|e| {
                        (e.timestamp - escalate_time).num_seconds().abs()
                            <= self.thresholds.priv_esc_window_secs
                    })
                    .collect();

                if !access_edges.is_empty() {
                    let entities: Vec<String> = vec!["user".to_string()];

                    let mut incident = Incident::new(&ChainType::PrivilegeEscalate, entities, 0.6);

                    let tactic_span_weight = 1.75;
                    let asset_crit = self.thresholds.get_asset_criticality("user:admin");
                    incident.base_signal *= tactic_span_weight * asset_crit;

                    info!("Detected privilege escalation: {}", incident.id);

                    return Some(incident);
                }
            }
        }

        None
    }

    pub fn add_incident(&self, incident: Incident) {
        let mut store = self.incident_store.write().unwrap();
        store.push_back(incident);
    }

    pub fn get_incidents(&self, severity: Option<&str>, limit: usize) -> Vec<Incident> {
        let store = self.incident_store.read().unwrap();

        store
            .get_all()
            .into_iter()
            .filter(|i| severity.map_or(true, |s| i.severity.to_uppercase() == s.to_uppercase()))
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn get_incident(&self, id: &str) -> Option<Incident> {
        let store = self.incident_store.read().unwrap();
        store.get_all().into_iter().find(|i| i.id == id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incident_scoring() {
        let thresholds = CorrelationThresholds::default();

        let base_signal = 0.8;
        let tactic_span_weight = 1.25;
        let asset_crit = thresholds.get_asset_criticality("user:admin");

        let final_score = base_signal * tactic_span_weight * asset_crit;

        assert!((final_score - 2.0).abs() < 0.01);
    }
}
