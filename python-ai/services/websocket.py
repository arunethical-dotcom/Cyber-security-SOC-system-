import asyncio
from typing import Set
from fastapi import WebSocket
from loguru import logger
import json


class WebSocketManager:
    def __init__(self):
        self._connections: Set[WebSocket] = set()

    async def start(self):
        logger.info("WebSocket manager started")

    async def stop(self):
        for conn in list(self._connections):
            await conn.close()
        self._connections.clear()
        logger.info("WebSocket manager stopped")

    async def connect(self, websocket: WebSocket):
        await websocket.accept()
        self._connections.add(websocket)
        logger.info(f"WebSocket connected. Total connections: {len(self._connections)}")

    def disconnect(self, websocket: WebSocket):
        self._connections.discard(websocket)
        logger.info(
            f"WebSocket disconnected. Total connections: {len(self._connections)}"
        )

    async def broadcast(self, message: dict):
        disconnected = set()

        for conn in self._connections:
            try:
                await conn.send_json(message)
            except Exception as e:
                logger.warning(f"Failed to send to WebSocket: {e}")
                disconnected.add(conn)

        for conn in disconnected:
            self._connections.discard(conn)

    async def send_to(self, websocket: WebSocket, message: dict):
        try:
            await websocket.send_json(message)
        except Exception as e:
            logger.warning(f"Failed to send to WebSocket: {e}")
            self._connections.discard(websocket)
