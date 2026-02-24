"""Pydantic event models for WebSocket communication."""

from enum import Enum
from typing import Literal

from pydantic import BaseModel


class EventType(str, Enum):
    """Types of events sent from server to client."""

    SCOUT_MOVE = "scout_move"
    EPISODE_COMPLETE = "episode_complete"
    TRAINING_UPDATE = "training_update"
    POLICY_UPDATE = "policy_update"
    TRAINING_COMPLETE = "training_complete"
    ERROR = "error"


class CommandType(str, Enum):
    """Types of commands sent from client to server."""

    START = "start"
    PAUSE = "pause"
    RESUME = "resume"
    SET_SPEED = "set_speed"
    STOP = "stop"


# Server -> Client Events


class ScoutMoveEvent(BaseModel):
    """Emitted when a scout takes a step."""

    type: Literal["scout_move"] = "scout_move"
    scout_id: str
    scout_index: int
    position: tuple[int, int]
    action: int
    reward: float
    done: bool
    step: int


class EpisodeCompleteEvent(BaseModel):
    """Emitted when a scout completes an episode."""

    type: Literal["episode_complete"] = "episode_complete"
    scout_id: str
    scout_index: int
    reached_goal: bool
    total_reward: float
    steps: int


class TrainingUpdateEvent(BaseModel):
    """Emitted after each training episode."""

    type: Literal["training_update"] = "training_update"
    episode: int
    total_episodes: int
    success_rate: float
    loss: float
    episode_reward: float


class PolicyUpdateEvent(BaseModel):
    """Emitted when the policy is updated."""

    type: Literal["policy_update"] = "policy_update"
    policy: list[list[int]]  # Grid of best actions per cell


class TrainingCompleteEvent(BaseModel):
    """Emitted when training finishes."""

    type: Literal["training_complete"] = "training_complete"
    final_success_rate: float
    history: dict[str, list[float]]


class ErrorEvent(BaseModel):
    """Emitted when an error occurs."""

    type: Literal["error"] = "error"
    message: str


# Client -> Server Commands


class TrainingConfig(BaseModel):
    """Configuration for training."""

    n_scouts: int = 5
    grid_size: int = 5
    episodes: int = 100
    steps_per_episode: int = 100
    with_obstacles: bool = False
    seed: int | None = None


class TrainingCommand(BaseModel):
    """Command from client to control training."""

    command: CommandType
    config: TrainingConfig | None = None
    speed: float | None = None  # For SET_SPEED command
