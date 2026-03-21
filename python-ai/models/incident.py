from datetime import datetime
from typing import Optional, List, Literal
from pydantic import BaseModel, Field


class Incident(BaseModel):
    id: str
    timestamp: datetime
    severity: str
    chain: List[str] = Field(default_factory=list)
    entities: List[str] = Field(default_factory=list)
    sigma_score: float = 0.0
    z_score: float = 0.0
    ioc_match: Optional[str] = None
    cvss: float = 0.0
    summary: Optional[str] = None
    actions: Optional[List[str]] = None


class Feedback(BaseModel):
    incident_id: str
    action: Literal["suppress", "tune", "confirm"]
    entity: Optional[str] = None
    tactic: Optional[str] = None
    new_threshold: Optional[float] = None
