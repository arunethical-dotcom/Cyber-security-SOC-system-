import asyncio
import time
from typing import Optional, Dict, Any
from loguru import logger


class IntelService:
    def __init__(self, ttl: int = 3600):
        self.ttl = ttl
        self._cache: Dict[str, tuple[Any, float]] = {}
        self._api_key: Optional[str] = None

    def set_api_key(self, api_key: str):
        self._api_key = api_key
        logger.info("OTX API key configured")

    async def enrich_ip(self, ip: str) -> Dict[str, Any]:
        if ip in self._cache:
            result, expires_at = self._cache[ip]
            if time.time() < expires_at:
                logger.debug(f"Cache hit for IP: {ip}")
                return result

        result = await self._fetch_ip_intel(ip)

        expires_at = time.time() + self.ttl
        self._cache[ip] = (result, expires_at)

        return result

    async def _fetch_ip_intel(self, ip: str) -> Dict[str, Any]:
        if not self._api_key:
            logger.debug(f"No OTX API key configured, returning empty result for {ip}")
            return self._empty_result(ip)

        try:
            import httpx

            async with httpx.AsyncClient() as client:
                response = await client.get(
                    f"https://otx.alienvault.com/api/v1/indicators/IPv4/{ip}/general",
                    headers={"X-OTX-API-KEY": self._api_key},
                    timeout=10.0,
                )

                if response.status_code == 200:
                    data = response.json()
                    return {
                        "ip": ip,
                        "pulse_count": data.get("pulse_count", 0),
                        "reputation": data.get("reputation", 0),
                        "country": data.get("country_code", ""),
                        "threats": data.get("threat_names", []),
                    }
                else:
                    logger.warning(f"OTX API returned {response.status_code} for {ip}")
                    return self._empty_result(ip)

        except Exception as e:
            logger.error(f"Failed to fetch OTX intel for {ip}: {e}")
            return self._empty_result(ip)

    def _empty_result(self, ip: str) -> Dict[str, Any]:
        return {
            "ip": ip,
            "pulse_count": 0,
            "reputation": 0,
            "country": "",
            "threats": [],
        }

    def clear_cache(self):
        self._cache.clear()
        logger.info("Intel cache cleared")
