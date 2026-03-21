import asyncio
from typing import Optional, List
import httpx
from loguru import logger

from models.incident import Incident
from models.feedback import Feedback


class RustClient:
    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url
        self.client: Optional[httpx.AsyncClient] = None
        self._poll_task: Optional[asyncio.Task] = None
        self._callbacks: List[callable] = []
        self._last_incident_ids: set = set()

    async def start(self):
        self.client = httpx.AsyncClient(base_url=self.base_url, timeout=30.0)
        logger.info(f"Rust client started with base URL: {self.base_url}")

    async def stop(self):
        if self._poll_task:
            self._poll_task.cancel()
            try:
                await self._poll_task
            except asyncio.CancelledError:
                pass
        if self.client:
            await self.client.aclose()
        logger.info("Rust client stopped")

    def on_new_incident(self, callback: callable):
        self._callbacks.append(callback)

    async def start_polling(self, interval: float = 2.0):
        async def poll():
            while True:
                try:
                    incidents = await self.get_incidents(limit=10)
                    current_ids = {inc.id for inc in incidents}

                    new_ids = current_ids - self._last_incident_ids
                    if new_ids:
                        for inc in incidents:
                            if inc.id in new_ids:
                                for cb in self._callbacks:
                                    try:
                                        await cb(inc)
                                    except Exception as e:
                                        logger.error(f"Callback error: {e}")

                    self._last_incident_ids = current_ids

                except Exception as e:
                    logger.error(f"Polling error: {e}")

                await asyncio.sleep(interval)

        self._poll_task = asyncio.create_task(poll())

    async def get_incidents(
        self, page: int = 1, limit: int = 50, severity: Optional[str] = None
    ) -> List[Incident]:
        params = {"page": page, "limit": limit}
        if severity:
            params["severity"] = severity

        response = await self.client.get("/incidents", params=params)
        response.raise_for_status()

        data = response.json()
        return [Incident(**item) for item in data]

    async def get_incident(self, id: str) -> Incident:
        response = await self.client.get(f"/incidents/{id}")
        response.raise_for_status()
        return Incident(**response.json())

    async def get_graph_snapshot(self) -> dict:
        response = await self.client.get("/graph/snapshot")
        response.raise_for_status()
        return response.json()

    async def post_feedback(self, feedback: Feedback) -> bool:
        response = await self.client.post("/feedback", json=feedback.model_dump())
        return response.status_code == 200

    async def health_check(self) -> bool:
        try:
            response = await self.client.get("/health")
            return response.status_code == 200
        except Exception:
            return False
