use rusqlite::{params, Connection};
use shared::CanonicalEvent;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tracing::{error, info};

const ALPHA: f32 = 0.1;

#[derive(Clone)]
pub struct EntityBaseline {
    pub ema_mean: f32,
    pub ema_std: f32,
    pub sample_count: u64,
    pub last_value: f32,
}

impl EntityBaseline {
    pub fn new(initial_value: f32) -> Self {
        Self {
            ema_mean: initial_value,
            ema_std: 1.0,
            sample_count: 1,
            last_value: initial_value,
        }
    }

    pub fn update(&mut self, value: f32) {
        let new_mean = ALPHA * value + (1.0 - ALPHA) * self.ema_mean;
        let new_std =
            (ALPHA * (value - new_mean).powi(2) + (1.0 - ALPHA) * self.ema_std.powi(2)).sqrt();

        self.ema_mean = new_mean;
        self.ema_std = new_std.max(0.1);
        self.sample_count += 1;
        self.last_value = value;
    }

    pub fn z_score(&self, value: f32) -> f32 {
        if self.ema_std == 0.0 {
            return 0.0;
        }
        (value - self.ema_mean) / self.ema_std
    }
}

pub struct BaselineStore {
    baselines: Arc<RwLock<HashMap<String, EntityBaseline>>>,
    db_path: String,
}

impl BaselineStore {
    pub fn new(db_path: impl Into<String>) -> Self {
        let db_path = db_path.into();
        let store = Self {
            baselines: Arc::new(RwLock::new(HashMap::new())),
            db_path: db_path.clone(),
        };

        if let Err(e) = store.load_from_db() {
            error!("Failed to load baselines from DB: {}", e);
        }

        store
    }

    fn get_db_connection(&self) -> Result<Connection, rusqlite::Error> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS baselines (
                entity_key TEXT PRIMARY KEY,
                ema_mean REAL NOT NULL,
                ema_std REAL NOT NULL,
                sample_count INTEGER NOT NULL,
                last_value REAL NOT NULL
            )",
            [],
        )?;
        Ok(conn)
    }

    pub fn load_from_db(&self) -> Result<(), rusqlite::Error> {
        let conn = self.get_db_connection()?;

        let mut stmt = conn.prepare(
            "SELECT entity_key, ema_mean, ema_std, sample_count, last_value FROM baselines",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                EntityBaseline {
                    ema_mean: row.get(1)?,
                    ema_std: row.get(2)?,
                    sample_count: row.get(3)?,
                    last_value: row.get(4)?,
                },
            ))
        })?;

        let mut baselines = self.baselines.write().unwrap();
        for row in rows {
            let (key, baseline) = row?;
            baselines.insert(key, baseline);
        }

        info!("Loaded {} baselines from database", baselines.len());
        Ok(())
    }

    pub fn persist_to_db(&self) -> Result<(), rusqlite::Error> {
        let conn = self.get_db_connection()?;

        let baselines = self.baselines.read().unwrap();

        for (key, baseline) in baselines.iter() {
            conn.execute(
                "INSERT OR REPLACE INTO baselines (entity_key, ema_mean, ema_std, sample_count, last_value)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    key,
                    baseline.ema_mean,
                    baseline.ema_std,
                    baseline.sample_count,
                    baseline.last_value,
                ],
            )?;
        }

        info!("Persisted {} baselines to database", baselines.len());
        Ok(())
    }

    pub fn get_baseline(&self, entity_key: &str) -> Option<EntityBaseline> {
        let baselines = self.baselines.read().unwrap();
        baselines.get(entity_key).cloned()
    }

    pub fn update_baseline(&self, entity_key: &str, value: f32) {
        let mut baselines = self.baselines.write().unwrap();

        if let Some(baseline) = baselines.get_mut(entity_key) {
            baseline.update(value);
        } else {
            baselines.insert(entity_key.to_string(), EntityBaseline::new(value));
        }
    }

    pub fn compute_z_score(&self, entity_key: &str, value: f32) -> f32 {
        let baselines = self.baselines.read().unwrap();

        if let Some(baseline) = baselines.get(entity_key) {
            baseline.z_score(value)
        } else {
            0.0
        }
    }

    pub fn normalize_z_score(z: f32) -> f32 {
        (z / 6.0).min(1.0).max(0.0)
    }

    pub fn record_event(&self, event: &CanonicalEvent) {
        let src_key = &event.src_entity.key;
        let dst_key = &event.dst_entity.key;

        self.update_baseline(src_key, 1.0);
        self.update_baseline(dst_key, 1.0);

        if let Some(bytes) = event.raw_fields.get("bytes") {
            if let Ok(byte_count) = bytes.parse::<f32>() {
                self.update_baseline(&format!("{}_bytes", src_key), byte_count);
            }
        }
    }

    pub fn get_all_baselines(&self) -> HashMap<String, EntityBaseline> {
        let baselines = self.baselines.read().unwrap();
        baselines.clone()
    }
}

impl Default for EntityBaseline {
    fn default() -> Self {
        Self::new(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema_update() {
        let mut baseline = EntityBaseline::new(10.0);

        baseline.update(12.0);
        assert!((baseline.ema_mean - 10.2).abs() < 0.01);

        baseline.update(14.0);
        assert!((baseline.ema_mean - 11.38).abs() < 0.01);
    }

    #[test]
    fn test_z_score() {
        let baseline = EntityBaseline {
            ema_mean: 10.0,
            ema_std: 2.0,
            sample_count: 10,
            last_value: 10.0,
        };

        assert!((baseline.z_score(14.0) - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_z_score_normalization() {
        assert!((BaselineStore::normalize_z_score(3.0) - 0.5).abs() < 0.01);
        assert!((BaselineStore::normalize_z_score(6.0) - 1.0).abs() < 0.01);
        assert!((BaselineStore::normalize_z_score(-3.0) - 0.0).abs() < 0.01);
    }
}
