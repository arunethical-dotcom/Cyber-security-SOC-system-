import httpx
import json
from typing import Optional
from loguru import logger
from pydantic import ValidationError

from models.incident import Incident
from models.llm_output import LLMOutput


class LLMService:
    def __init__(
        self, ollama_url: str = "http://localhost:11434", model: str = "qwen:1.8b"
    ):
        self.ollama_url = ollama_url
        self.model = model
        self.client = httpx.AsyncClient(timeout=120.0)

    async def generate_summary(self, incident: Incident) -> LLMOutput:
        prompt = self._build_prompt(incident)

        try:
            response = await self.client.post(
                f"{self.ollama_url}/api/generate",
                json={
                    "model": self.model,
                    "prompt": prompt,
                    "format": "json",
                    "stream": False,
                },
            )

            if response.status_code != 200:
                logger.warning(
                    f"Ollama returned {response.status_code}, using fallback"
                )
                return self._get_fallback(incident)

            result = response.json()
            llm_text = result.get("response", "")

            try:
                return LLMOutput.model_validate_json(llm_text)
            except ValidationError as e:
                logger.warning(f"LLM output validation failed: {e}, using fallback")
                return self._get_fallback(incident)

        except Exception as e:
            logger.error(f"LLM call failed: {e}, using fallback")
            return self._get_fallback(incident)

    def _build_prompt(self, incident: Incident) -> str:
        incident_json = incident.model_dump_json()

        prompt = f"""You are a security analyst writing a structured incident report.
Analyze this security incident and respond ONLY with valid JSON
matching the exact schema provided. Do not add any text outside the JSON.

Schema: {{ incident_type, severity, summary, recommended_actions, confidence_caveat }}

Incident data:
{incident_json}

Respond with JSON only."""

        return prompt

    def _get_fallback(self, incident: Incident) -> LLMOutput:
        severity_map = {
            "critical": "CRITICAL",
            "high": "HIGH",
            "medium": "MEDIUM",
            "low": "LOW",
        }

        return LLMOutput(
            incident_type="security_incident",
            severity=severity_map.get(incident.severity.lower(), "MEDIUM"),
            summary=f"Security incident detected with CVSS {incident.cvss}. Chain: {', '.join(incident.chain)}",
            recommended_actions=[
                "Investigate affected entities",
                "Review logs for related activity",
                "Implement containment measures if needed",
            ],
            confidence_caveat="Automated analysis - manual review recommended",
        )

    async def close(self):
        await self.client.aclose()
