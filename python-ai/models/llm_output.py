from typing import List, Literal
from pydantic import BaseModel, Field


class LLMOutput(BaseModel):
    incident_type: str
    severity: Literal["LOW", "MEDIUM", "HIGH", "CRITICAL"]
    summary: str
    recommended_actions: List[str] = Field(default_factory=list)
    confidence_caveat: str = ""
