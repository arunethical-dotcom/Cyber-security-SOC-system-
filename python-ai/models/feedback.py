from typing import Optional, Literal
from pydantic import BaseModel


class Feedback(BaseModel):
    incident_id: str
    action: Literal["suppress", "tune", "confirm"]
    entity: Optional[str] = None
    tactic: Optional[str] = None
    new_threshold: Optional[float] = None
