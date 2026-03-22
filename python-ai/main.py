from contextlib import asynccontextmanager
from fastapi import FastAPI
from loguru import logger

from routers import incidents, reports, feedback, graph, stream
from services.rust_client import RustClient
from services.intel import IntelService
from services.websocket import WebSocketManager
from services.anomaly import get_anomaly_detector
from services.siem import init_db, ingest_incident
from routers.siem import router as siem_router


rust_client: RustClient = None
intel_service: IntelService = None
ws_manager: WebSocketManager = None


async def on_new_incident(incident):
    """
    Callback when a new incident is received from Rust backend.
    Enriches with anomaly score, broadcasts to WebSocket, and persists to SIEM.
    """
    try:
        # Convert incident to dict if needed
        enriched_incident = (
            incident.model_dump() if hasattr(incident, "model_dump") else incident
        )

        # R3 — Calculate anomaly score and feed into model for retraining
        anomaly_detector = get_anomaly_detector()
        anomaly_score = anomaly_detector.score(enriched_incident)
        enriched_incident["anomaly_score"] = anomaly_score
        anomaly_detector.record_event(enriched_incident)

        # Broadcast to WebSocket clients
        await ws_manager.broadcast(enriched_incident)

        # R5 — Persist to SIEM
        ingest_incident(enriched_incident)

        logger.info(f"Processed incident {enriched_incident.get('id')}")
    except Exception as e:
        logger.error(f"Error processing incident: {e}")


@asynccontextmanager
async def lifespan(app: FastAPI):
    global rust_client, intel_service, ws_manager

    logger.info("Starting Python AI Layer")

    # R5 — Initialize SIEM database
    init_db()

    # R3 — Initialize anomaly detector
    get_anomaly_detector()

    rust_client = RustClient()
    await rust_client.start()

    # Register callback for new incidents
    rust_client.on_new_incident(on_new_incident)
    await rust_client.start_polling(interval=2.0)

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
app.include_router(siem_router)


@app.get("/health")
async def health():
    return {"status": "ok"}
