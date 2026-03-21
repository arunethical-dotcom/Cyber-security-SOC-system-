from typing import List, Optional
from fastapi import APIRouter, Query
from models.incident import Incident

router = APIRouter()


@router.get("", response_model=List[Incident])
async def get_incidents(
    page: int = Query(1, ge=1),
    limit: int = Query(50, ge=1, le=100),
    severity: Optional[str] = None,
):
    return []


@router.get("/{incident_id}", response_model=Incident)
async def get_incident(incident_id: str):
    return {
        "id": incident_id,
        "timestamp": "2024-01-01T00:00:00Z",
        "severity": "HIGH",
        "chain": ["T1110", "TA0001"],
        "entities": ["user:admin"],
        "sigma_score": 0.8,
        "z_score": 2.5,
        "cvss": 7.5,
    }
