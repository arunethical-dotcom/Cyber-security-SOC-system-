"""
R5 — SIEM Database Service
Persists incidents to SQLite with search and analytics capabilities
"""

import sqlite3
import json
from datetime import datetime
from typing import List, Dict, Optional
from loguru import logger

DB_PATH = "siem.db"


def init_db():
    """Initialize database schema"""
    try:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        # Create incidents table
        cursor.execute(
            """
            CREATE TABLE IF NOT EXISTS incidents (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                severity TEXT NOT NULL,
                sigma_score REAL,
                z_score REAL,
                cvss REAL,
                ioc_match TEXT,
                chain TEXT,
                entities TEXT,
                summary TEXT,
                actions TEXT,
                ingested_at TEXT NOT NULL
            )
            """
        )

        # Create indexes
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_severity ON incidents(severity)")
        cursor.execute("CREATE INDEX IF NOT EXISTS idx_timestamp ON incidents(timestamp)")

        conn.commit()
        conn.close()
        logger.info("R5: SIEM database ready")
    except Exception as e:
        logger.error(f"R5: Failed to initialize database: {e}")


def ingest_incident(incident: dict):
    """
    Insert or replace an incident in the database.
    Serializes complex fields (chain, entities, actions) as JSON strings.
    """
    try:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        # Serialize complex fields
        chain_json = json.dumps(incident.get("chain", []))
        entities_json = json.dumps(incident.get("entities", []))
        actions_json = json.dumps(incident.get("actions", []))

        # Get current timestamp
        ingested_at = datetime.utcnow().isoformat()

        cursor.execute(
            """
            INSERT OR REPLACE INTO incidents 
            (id, timestamp, severity, sigma_score, z_score, cvss, ioc_match, 
             chain, entities, summary, actions, ingested_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                incident.get("id"),
                incident.get("timestamp"),
                incident.get("severity"),
                incident.get("sigma_score"),
                incident.get("z_score"),
                incident.get("cvss"),
                incident.get("ioc_match"),
                chain_json,
                entities_json,
                incident.get("summary"),
                actions_json,
                ingested_at,
            ),
        )

        conn.commit()
        conn.close()
    except Exception as e:
        logger.error(f"R5: Failed to ingest incident: {e}")


def search_incidents(
    severity: Optional[str] = None,
    from_date: Optional[str] = None,
    to_date: Optional[str] = None,
    entity: Optional[str] = None,
    limit: int = 50,
    offset: int = 0,
) -> List[Dict]:
    """
    Search incidents with optional filters.
    Builds WHERE clause dynamically from provided parameters.
    Entity search uses LIKE %entity% pattern matching on JSON field.
    Returns list of dicts with deserialized complex fields.
    """
    try:
        conn = sqlite3.connect(DB_PATH)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()

        # Build WHERE clause dynamically
        where_clauses = []
        params = []

        if severity:
            where_clauses.append("severity = ?")
            params.append(severity)

        if from_date:
            where_clauses.append("timestamp >= ?")
            params.append(from_date)

        if to_date:
            where_clauses.append("timestamp <= ?")
            params.append(to_date)

        if entity:
            where_clauses.append("entities LIKE ?")
            params.append(f"%{entity}%")

        # Build final query
        where_clause = " AND ".join(where_clauses) if where_clauses else "1=1"
        query = f"SELECT * FROM incidents WHERE {where_clause} ORDER BY timestamp DESC LIMIT ? OFFSET ?"
        params.extend([limit, offset])

        cursor.execute(query, params)
        rows = cursor.fetchall()
        conn.close()

        # Convert rows to dicts and deserialize JSON
        results = []
        for row in rows:
            incident_dict = dict(row)
            incident_dict["chain"] = json.loads(incident_dict.get("chain", "[]"))
            incident_dict["entities"] = json.loads(incident_dict.get("entities", "[]"))
            incident_dict["actions"] = json.loads(incident_dict.get("actions", "[]"))
            results.append(incident_dict)

        return results
    except Exception as e:
        logger.error(f"R5: Search failed: {e}")
        return []


def get_stats() -> Dict:
    """
    Get summary statistics about incidents.
    Returns dict with total_incidents, last_hour count, by_severity, and top_entities.
    """
    try:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        # Total incidents
        cursor.execute("SELECT COUNT(*) as total FROM incidents")
        total_incidents = cursor.fetchone()[0]

        # Last hour count
        cursor.execute(
            """
            SELECT COUNT(*) as count FROM incidents 
            WHERE ingested_at >= datetime('now', '-1 hour')
            """
        )
        last_hour_count = cursor.fetchone()[0]

        # By severity
        cursor.execute(
            """
            SELECT severity, COUNT(*) as count FROM incidents 
            GROUP BY severity
            """
        )
        by_severity = {row[0]: row[1] for row in cursor.fetchall()}

        # Top entities (requires parsing JSON)
        cursor.execute("SELECT entities FROM incidents")
        entity_counts = {}
        for row in cursor.fetchall():
            entities = json.loads(row[0] or "[]")
            for entity in entities:
                entity_counts[entity] = entity_counts.get(entity, 0) + 1

        # Top 5
        top_entities = dict(sorted(entity_counts.items(), key=lambda x: x[1], reverse=True)[:5])

        conn.close()

        return {
            "total_incidents": total_incidents,
            "last_hour_count": last_hour_count,
            "by_severity": by_severity,
            "top_entities": top_entities,
        }
    except Exception as e:
        logger.error(f"R5: Stats query failed: {e}")
        return {
            "total_incidents": 0,
            "last_hour_count": 0,
            "by_severity": {},
            "top_entities": {},
        }


def get_timeline(hours: int = 24) -> List[Dict]:
    """
    Get incident timeline grouped by hour and severity.
    Returns list of dicts with timestamp (ISO format, hourly) and severity counts.
    """
    try:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        # Query grouped by hour and severity
        cursor.execute(
            """
            SELECT 
                strftime('%Y-%m-%dT%H:00:00', timestamp) as hour,
                severity,
                COUNT(*) as count
            FROM incidents
            WHERE timestamp >= datetime('now', '-' || ? || ' hours')
            GROUP BY hour, severity
            ORDER BY hour
            """,
            (hours,),
        )

        results = []
        for row in cursor.fetchall():
            results.append({"timestamp": row[0], "severity": row[1], "count": row[2]})

        conn.close()
        return results
    except Exception as e:
        logger.error(f"R5: Timeline query failed: {e}")
        return []


def delete_incident(incident_id: str) -> bool:
    """
    Delete an incident by ID.
    Returns True if deletion successful (rowcount > 0), False otherwise.
    """
    try:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        cursor.execute("DELETE FROM incidents WHERE id = ?", (incident_id,))
        conn.commit()

        deleted = cursor.rowcount > 0
        conn.close()

        return deleted
    except Exception as e:
        logger.error(f"R5: Delete failed: {e}")
        return False
