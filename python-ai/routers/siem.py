"""
R5 — SIEM Router
FastAPI endpoints for SIEM queries and incident management
"""

from fastapi import APIRouter, HTTPException, Query
from typing import Optional
from loguru import logger

from services.siem import (
    search_incidents,
    get_stats,
    get_timeline,
    delete_incident,
)

router = APIRouter(prefix="/siem", tags=["siem"])


@router.get("/search")
async def search_siem(
    severity: Optional[str] = Query(None),
    from_date: Optional[str] = Query(None),
    to_date: Optional[str] = Query(None),
    entity: Optional[str] = Query(None),
    limit: int = Query(50, ge=1, le=500),
    offset: int = Query(0, ge=0),
):
    """
    Search incidents with optional filters.
    - severity: Filter by severity (LOW, MEDIUM, HIGH, CRITICAL)
    - from_date: Filter incidents after this date (ISO format)
    - to_date: Filter incidents before this date (ISO format)
    - entity: Search for incidents involving this entity (LIKE pattern)
    - limit: Max results (1-500)
    - offset: Pagination offset

    Returns list of incidents with all fields.
    """
    try:
        incidents = search_incidents(
            severity=severity,
            from_date=from_date,
            to_date=to_date,
            entity=entity,
            limit=limit,
            offset=offset,
        )
        return incidents
    except Exception as e:
        logger.error(f"R5: Search error: {e}")
        raise HTTPException(status_code=500, detail="Search failed")


@router.get("/stats")
async def get_siem_stats():
    """
    Get SIEM summary statistics.
    Returns:
    - total_incidents: Total incident count
    - last_hour_count: Incidents in last hour
    - by_severity: Dict of severity -> count
    - top_entities: Top 5 entities by incident count
    """
    try:
        stats = get_stats()
        return stats
    except Exception as e:
        logger.error(f"R5: Stats error: {e}")
        raise HTTPException(status_code=500, detail="Stats retrieval failed")


@router.get("/timeline")
async def get_siem_timeline(hours: int = Query(24, ge=1, le=168)):
    """
    Get incident timeline grouped by hour and severity.
    - hours: Number of hours to look back (1-168)

    Returns list of dicts with timestamp (hourly ISO format), severity, and count.
    """
    try:
        timeline = get_timeline(hours=hours)
        return timeline
    except Exception as e:
        logger.error(f"R5: Timeline error: {e}")
        raise HTTPException(status_code=500, detail="Timeline retrieval failed")


@router.delete("/incidents/{incident_id}")
async def delete_siem_incident(incident_id: str):
    """
    Delete an incident by ID.
    Returns 404 if incident not found.
    """
    try:
        success = delete_incident(incident_id)
        if not success:
            raise HTTPException(status_code=404, detail="Incident not found")
        return {"success": True, "incident_id": incident_id}
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"R5: Delete error: {e}")
        raise HTTPException(status_code=500, detail="Delete failed")
