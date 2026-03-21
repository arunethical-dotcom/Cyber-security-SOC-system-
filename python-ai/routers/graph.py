from fastapi import APIRouter

router = APIRouter()


@router.get("/snapshot")
async def get_graph_snapshot():
    return {"nodes": [], "edges": []}
