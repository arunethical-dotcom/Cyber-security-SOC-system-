use chrono::{DateTime, Utc};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::EdgeType;
use serde::{Deserialize, Serialize};
use shared::{CanonicalEvent, Entity, EntityKind, EventType};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::info;

const GRAPH_TTL_SECS: i64 = 1800;
const PRUNE_INTERVAL_SECS: u64 = 60;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityNode {
    pub key: String,
    pub kind: EntityKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventEdge {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub score: f32,
    pub sigma_rule_id: Option<String>,
}

pub struct EventGraph {
    graph: DiGraph<EntityNode, EventEdge>,
    node_index_map: HashMap<String, NodeIndex>,
    pruner_handle: Option<tokio::task::JoinHandle<()>>,
}

impl EventGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_index_map: HashMap::new(),
            pruner_handle: None,
        }
    }

    pub fn start_pruner(graph: Arc<RwLock<Self>>) {
        let graph_clone = graph.clone();
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(PRUNE_INTERVAL_SECS)
            );
            
            loop {
                interval.tick().await;
                
                let mut g = graph_clone.write().unwrap();
                g.prune_old_edges();
            }
        });
        
        let mut graph = graph.write().unwrap();
        graph.pruner_handle = Some(handle);
    }

    pub fn insert_event(&mut self, event: &CanonicalEvent, score: f32, sigma_rule_id: Option<String>) {
        let src_key = event.src_entity.key.clone();
        let dst_key = event.dst_entity.key.clone();
        
        let src_idx = self.get_or_insert_node(&src_key, event.src_entity.kind.clone());
        let dst_idx = self.get_or_insert_node(&dst_key, event.dst_entity.kind.clone());
        
        let edge = EventEdge {
            id: event.id.to_string(),
            timestamp: event.timestamp,
            event_type: event.event_type,
            score,
            sigma_rule_id,
        };
        
        self.graph.add_edge(src_idx, dst_idx, edge);
        
        tracing::debug!(
            "Inserted edge from {} to {} with score {}",
            src_key, dst_key, score
        );
    }

    fn get_or_insert_node(&mut self, key: &str, kind: EntityKind) -> NodeIndex {
        if let Some(&idx) = self.node_index_map.get(key) {
            return idx;
        }
        
        let node = EntityNode {
            key: key.to_string(),
            kind,
        };
        
        let idx = self.graph.add_node(node);
        self.node_index_map.insert(key.to_string(), idx);
        
        tracing::debug!("Added node {} at index {:?}", key, idx);
        
        idx
    }

    pub fn prune_old_edges(&mut self) {
        let now = Utc::now();
        let mut edges_to_remove = Vec::new();
        
        for edge_idx in self.graph.edge_indices() {
            let edge = &self.graph[edge_idx];
            let elapsed = (now - edge.timestamp).num_seconds();
            
            if elapsed > GRAPH_TTL_SECS {
                edges_to_remove.push(edge_idx);
            }
        }
        
        let removed_count = edges_to_remove.len();
        for edge_idx in edges_to_remove {
            self.graph.remove_edge(edge_idx);
        }
        
        self.prune_orphaned_nodes();
        
        if removed_count > 0 {
            info!("Pruned {} old edges from graph", removed_count);
        }
    }

    fn prune_orphaned_nodes(&mut self) {
        let mut nodes_to_remove = Vec::new();
        
        for node_idx in self.graph.node_indices() {
            let degree = self.graph.edges(node_idx).count();
            if degree == 0 {
                nodes_to_remove.push(node_idx);
            }
        }
        
        let removed_count = nodes_to_remove.len();
        for node_idx in nodes_to_remove {
            let node = self.graph.remove_node(node_idx);
            if let Some(EntityNode { key, .. }) = node {
                self.node_index_map.remove(&key);
            }
        }
        
        if removed_count > 0 {
            info!("Pruned {} orphaned nodes from graph", removed_count);
        }
    }

    pub fn get_snapshot(&self) -> GraphSnapshot {
        let nodes: Vec<GraphNodeDto> = self.graph
            .node_indices()
            .map(|idx| {
                let node = &self.graph[idx];
                GraphNodeDto {
                    id: idx.index(),
                    key: node.key.clone(),
                    kind: format!("{:?}", node.kind),
                }
            })
            .collect();
        
        let edges: Vec<GraphEdgeDto> = self.graph
            .edge_indices()
            .map(|idx| {
                let (src, dst) = self.graph.edge_endpoints(idx).unwrap();
                let edge = &self.graph[idx];
                GraphEdgeDto {
                    id: edge.id.clone(),
                    source: src.index(),
                    target: dst.index(),
                    event_type: format!("{:?}", edge.event_type),
                    score: edge.score,
                    timestamp: edge.timestamp.to_rfc3339(),
                }
            })
            .collect();
        
        GraphSnapshot { nodes, edges }
    }

    pub fn get_edges_for_entity(&self, entity_key: &str) -> Vec<&EventEdge> {
        if let Some(&idx) = self.node_index_map.get(entity_key) {
            self.graph
                .edges(idx)
                .map(|e| e.weight())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn find_chain(&self, pattern: &[EventType], window_secs: i64) -> Vec<Vec<&EventEdge>> {
        let now = Utc::now();
        let mut chains = Vec::new();
        
        for node_idx in self.graph.node_indices() {
            let edges: Vec<_> = self.graph
                .edges(node_idx)
                .map(|e| e.weight())
                .filter(|e| (now - e.timestamp).num_seconds() <= window_secs)
                .collect();
            
            self.find_chain_recursive(&edges, pattern, &mut Vec::new(), &mut chains);
        }
        
        chains
    }

    fn find_chain_recursive<'a>(
        &self,
        edges: &[&'a EventEdge],
        pattern: &[EventType],
        current: &mut Vec<&'a EventEdge>,
        results: &mut Vec<Vec<&'a EventEdge>>,
    ) {
        if pattern.is_empty() {
            if !current.is_empty() {
                results.push(current.clone());
            }
            return;
        }
        
        let target_type = &pattern[0];
        
        for edge in edges {
            if &edge.event_type == target_type {
                current.push(edge);
                self.find_chain_recursive(edges, &pattern[1..], current, results);
                current.pop();
            }
        }
    }
}

impl Default for EventGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphSnapshot {
    pub nodes: Vec<GraphNodeDto>,
    pub edges: Vec<GraphEdgeDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphNodeDto {
    pub id: usize,
    pub key: String,
    pub kind: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphEdgeDto {
    pub id: String,
    pub source: usize,
    pub target: usize,
    pub event_type: String,
    pub score: f32,
    pub timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_snapshot() {
        let mut graph = EventGraph::new();
        
        let event = CanonicalEvent::new(
            Entity::user("alice"),
            Entity::device("server1"),
            EventType::Login,
            HashMap::new(),
        );
        
        graph.insert_event(&event, 0.8, Some("test_rule".to_string()));
        
        let snapshot = graph.get_snapshot();
        assert_eq!(snapshot.nodes.len(), 2);
        assert_eq!(snapshot.edges.len(), 1);
    }

    #[test]
    fn test_node_deduplication() {
        let mut graph = EventGraph::new();
        
        let event1 = CanonicalEvent::new(
            Entity::user("alice"),
            Entity::device("server1"),
            EventType::Login,
            HashMap::new(),
        );
        
        let event2 = CanonicalEvent::new(
            Entity::user("alice"),
            Entity::device("server2"),
            EventType::Connect,
            HashMap::new(),
        );
        
        graph.insert_event(&event1, 0.5, None);
        graph.insert_event(&event2, 0.6, None);
        
        let snapshot = graph.get_snapshot();
        assert_eq!(snapshot.nodes.len(), 3);
    }
}
