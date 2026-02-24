"""FastAPI application for Many-Eyes Learning visualization."""

import sys
from pathlib import Path

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles

# Add project root to path for many_eyes imports
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root / "src"))

from .routes import experiments_router, training_router

app = FastAPI(
    title="Many-Eyes Learning Visualization",
    description="Real-time visualization of multi-scout reinforcement learning",
    version="0.1.0",
)

# Configure CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include routers
app.include_router(experiments_router)
app.include_router(training_router)

# Frontend static files
static_dir = Path(__file__).parent.parent / "frontend" / "dist"


@app.get("/api/grid-info")
async def get_grid_info():
    """Get default grid configuration."""
    return {
        "default_size": 5,
        "actions": {
            0: {"name": "up", "arrow": "\u2191"},
            1: {"name": "right", "arrow": "\u2192"},
            2: {"name": "down", "arrow": "\u2193"},
            3: {"name": "left", "arrow": "\u2190"},
        },
        "scout_colors": [
            {"id": 0, "name": "Scout 0", "color": "#2196F3"},
            {"id": 1, "name": "Scout 1", "color": "#4CAF50"},
            {"id": 2, "name": "Scout 2", "color": "#FF9800"},
            {"id": 3, "name": "Scout 3", "color": "#9C27B0"},
            {"id": 4, "name": "Scout 4", "color": "#F44336"},
        ],
    }


@app.get("/api/health")
async def health():
    """Health check endpoint."""
    return {"status": "ok", "version": "0.1.0"}


# Serve frontend - must be last to not override API routes
if static_dir.exists():
    app.mount("/static", StaticFiles(directory=static_dir), name="static")

    @app.get("/")
    async def serve_index():
        """Serve the frontend index.html."""
        return FileResponse(static_dir / "index.html")

    @app.get("/{path:path}")
    async def serve_frontend(path: str):
        """Serve frontend static files."""
        file_path = static_dir / path
        if file_path.exists() and file_path.is_file():
            return FileResponse(file_path)
        # Fallback to index.html for SPA routing
        return FileResponse(static_dir / "index.html")
else:

    @app.get("/")
    async def root():
        """API root when frontend not built."""
        return {
            "status": "ok",
            "name": "Many-Eyes Learning API",
            "message": "Frontend not built. Run: cd web/frontend && trunk build",
        }


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=3200)
