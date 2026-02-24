"""REST API routes for experiments."""

import json
from pathlib import Path

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

router = APIRouter(prefix="/api/experiments", tags=["experiments"])

# Path to experiments results
RESULTS_DIR = Path(__file__).parent.parent.parent.parent / "experiments" / "results"


class ExperimentSummary(BaseModel):
    """Summary of an experiment."""

    id: str
    name: str
    n_scouts: int
    episodes: int
    final_success_rate: float
    created_at: str | None = None


class ExperimentDetail(BaseModel):
    """Full experiment data for replay."""

    id: str
    name: str
    config: dict
    history: dict
    grid_size: int
    walls: list[list[int]]


@router.get("/", response_model=list[ExperimentSummary])
async def list_experiments():
    """List all saved experiments."""
    experiments = []

    if not RESULTS_DIR.exists():
        return experiments

    for file_path in RESULTS_DIR.glob("*.json"):
        try:
            with open(file_path) as f:
                data = json.load(f)

            # Handle different result formats
            if "results" in data:
                # Multi-experiment results file
                for name, result in data.get("results", {}).items():
                    if isinstance(result, dict):
                        experiments.append(
                            ExperimentSummary(
                                id=f"{file_path.stem}_{name}",
                                name=name,
                                n_scouts=result.get("n_scouts", 1),
                                episodes=len(result.get("history", {}).get("success_rates", [])),
                                final_success_rate=result.get("mean_success_rate", 0),
                            )
                        )
            else:
                # Single experiment file
                history = data.get("history", {})
                success_rates = history.get("success_rates", [])
                experiments.append(
                    ExperimentSummary(
                        id=file_path.stem,
                        name=data.get("name", file_path.stem),
                        n_scouts=data.get("n_scouts", 1),
                        episodes=len(success_rates),
                        final_success_rate=success_rates[-1] if success_rates else 0,
                    )
                )
        except (json.JSONDecodeError, KeyError):
            continue

    return experiments


@router.get("/{experiment_id}", response_model=ExperimentDetail)
async def get_experiment(experiment_id: str):
    """Get full experiment data for replay."""
    # Check if it's a compound ID (file_name_experiment_name)
    if "_" in experiment_id:
        parts = experiment_id.rsplit("_", 1)
        file_stem = parts[0]
        exp_name = parts[1] if len(parts) > 1 else None

        file_path = RESULTS_DIR / f"{file_stem}.json"
        if file_path.exists():
            with open(file_path) as f:
                data = json.load(f)

            if "results" in data and exp_name in data["results"]:
                result = data["results"][exp_name]
                return ExperimentDetail(
                    id=experiment_id,
                    name=exp_name,
                    config=data.get("config", {}),
                    history=result.get("history", {}),
                    grid_size=data.get("config", {}).get("grid_size", 5),
                    walls=data.get("config", {}).get("walls", []),
                )

    # Try as simple file name
    file_path = RESULTS_DIR / f"{experiment_id}.json"
    if file_path.exists():
        with open(file_path) as f:
            data = json.load(f)

        return ExperimentDetail(
            id=experiment_id,
            name=data.get("name", experiment_id),
            config=data.get("config", {}),
            history=data.get("history", {}),
            grid_size=data.get("config", {}).get("grid_size", 5),
            walls=data.get("config", {}).get("walls", []),
        )

    raise HTTPException(status_code=404, detail="Experiment not found")
