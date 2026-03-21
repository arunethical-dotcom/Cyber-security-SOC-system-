import os
from datetime import datetime
from pathlib import Path
from jinja2 import Environment, FileSystemLoader, select_autoescape
from loguru import logger

from models.incident import Incident
from models.llm_output import LLMOutput


class ReportService:
    def __init__(self, templates_dir: str = "templates", output_dir: str = "reports"):
        self.templates_dir = templates_dir
        self.output_dir = output_dir

        os.makedirs(output_dir, exist_ok=True)

        self.jinja_env = Environment(
            loader=FileSystemLoader(templates_dir),
            autoescape=select_autoescape(["html", "xml"]),
        )

    async def generate_report(self, incident: Incident, llm_output: LLMOutput) -> str:
        try:
            template = self.jinja_env.get_template("report.html.j2")

            html_content = template.render(
                incident=incident,
                llm_output=llm_output,
                generated_at=datetime.now().isoformat(),
            )

            filename = (
                f"report_{incident.id}_{datetime.now().strftime('%Y%m%d_%H%M%S')}.html"
            )
            filepath = os.path.join(self.output_dir, filename)

            with open(filepath, "w", encoding="utf-8") as f:
                f.write(html_content)

            logger.info(f"Generated report: {filepath}")
            return filepath

        except Exception as e:
            logger.error(f"Failed to generate report: {e}")
            return self._generate_fallback_markdown(incident, llm_output)

    def _generate_fallback_markdown(
        self, incident: Incident, llm_output: LLMOutput
    ) -> str:
        filename = f"report_{incident.id}_{datetime.now().strftime('%Y%m%d_%H%M%S')}.md"
        filepath = os.path.join(self.output_dir, filename)

        content = f"""# Security Incident Report

**Incident ID:** {incident.id}
**Timestamp:** {incident.timestamp.isoformat()}
**Severity:** {incident.severity}
**CVSS:** {incident.cvss}

## MITRE ATT&CK Chain
{chr(10).join(f"- {t}" for t in incident.chain)}

## Affected Entities
{chr(10).join(f"- {e}" for e in incident.entities)}

## Signal Scores
- Sigma Score: {incident.sigma_score}
- Z-Score: {incident.z_score}
- IOC Match: {incident.ioc_match or "None"}

## LLM Analysis

**Type:** {llm_output.incident_type}
**Severity:** {llm_output.severity}

### Summary
{llm_output.summary}

### Recommended Actions
{chr(10).join(f"{i + 1}. {a}" for i, a in enumerate(llm_output.recommended_actions))}

### Confidence Caveat
{llm_output.confidence_caveat}

---
Generated: {datetime.now().isoformat()}
"""

        with open(filepath, "w", encoding="utf-8") as f:
            f.write(content)

        logger.info(f"Generated fallback markdown report: {filepath}")
        return filepath
