from fastapi import APIRouter, HTTPException
from models.feedback import Feedback

router = APIRouter()


@router.post("")
async def submit_feedback(feedback: Feedback):
    return {
        "status": "ok",
        "message": f"Feedback received for incident {feedback.incident_id}",
    }
