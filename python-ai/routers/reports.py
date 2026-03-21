from fastapi import APIRouter, HTTPException
from fastapi.responses import FileResponse

router = APIRouter()


@router.get("/{report_id}")
async def get_report(report_id: str):
    return {"message": "Report generation not yet implemented"}


@router.get("/{report_id}/pdf")
async def get_report_pdf(report_id: str):
    raise HTTPException(status_code=404, detail="Report not found")
