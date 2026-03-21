from contextlib import asynccontextmanager
from fastapi import FastAPI
from loguru import logger

from routers import incidents, reports, feedback, graph, stream
from services.rust_client import RustClient
from services.intel import IntelService
from services.websocket import WebSocketManager


rust_client: RustClient = None
intel_service: IntelService = None
ws_manager: WebSocketManager = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    global rust_client, intel_service, ws_manager

    logger.info("Starting Python AI Layer")

    rust_client = RustClient()
    await rust_client.start()

    intel_service = IntelService()

    ws_manager = WebSocketManager()
    await ws_manager.start()

    yield

    logger.info("Shutting down Python AI Layer")
    await rust_client.stop()
    await ws_manager.stop()


app = FastAPI(title="SOC AI Layer", lifespan=lifespan)

app.include_router(incidents.router, prefix="/incidents", tags=["incidents"])
app.include_router(reports.router, prefix="/reports", tags=["reports"])
app.include_router(feedback.router, prefix="/feedback", tags=["feedback"])
app.include_router(graph.router, prefix="/graph", tags=["graph"])
app.include_router(stream.router, prefix="/stream", tags=["stream"])


@app.get("/health")
async def health():
    return {"status": "ok"}
