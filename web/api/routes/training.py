"""WebSocket route for real-time training."""

import asyncio
import json

from fastapi import APIRouter, WebSocket, WebSocketDisconnect

from ..models.events import CommandType, TrainingCommand, TrainingConfig
from ..services.streaming_trainer import StreamingTrainer

router = APIRouter()


class ConnectionManager:
    """Manages WebSocket connections."""

    def __init__(self):
        self.active_connections: dict[str, WebSocket] = {}
        self.trainers: dict[str, StreamingTrainer] = {}
        self.training_tasks: dict[str, asyncio.Task] = {}

    async def connect(self, websocket: WebSocket, client_id: str):
        """Accept a new connection."""
        await websocket.accept()
        self.active_connections[client_id] = websocket

    def disconnect(self, client_id: str):
        """Handle disconnection."""
        if client_id in self.active_connections:
            del self.active_connections[client_id]
        if client_id in self.trainers:
            self.trainers[client_id].stop()
            del self.trainers[client_id]
        if client_id in self.training_tasks:
            self.training_tasks[client_id].cancel()
            del self.training_tasks[client_id]

    async def send_event(self, client_id: str, event: dict):
        """Send an event to a client."""
        if client_id in self.active_connections:
            await self.active_connections[client_id].send_json(event)


manager = ConnectionManager()


async def run_training(client_id: str, config: TrainingConfig):
    """Run training and send events."""
    trainer = StreamingTrainer(config=config)
    manager.trainers[client_id] = trainer

    try:
        async for event in trainer.train():
            await manager.send_event(client_id, event.model_dump())
    except asyncio.CancelledError:
        pass
    finally:
        if client_id in manager.trainers:
            del manager.trainers[client_id]


@router.websocket("/ws/train/{client_id}")
async def training_websocket(websocket: WebSocket, client_id: str):
    """
    WebSocket endpoint for real-time training.

    Commands:
    - start: Start training with config
    - pause: Pause training
    - resume: Resume training
    - set_speed: Change training speed
    - stop: Stop training
    """
    await manager.connect(websocket, client_id)

    try:
        while True:
            data = await websocket.receive_text()
            try:
                message = json.loads(data)
                command = TrainingCommand(**message)
            except (json.JSONDecodeError, ValueError) as e:
                await websocket.send_json({"type": "error", "message": str(e)})
                continue

            if command.command == CommandType.START:
                # Stop any existing training
                if client_id in manager.training_tasks:
                    manager.training_tasks[client_id].cancel()

                # Start new training
                config = command.config or TrainingConfig()
                task = asyncio.create_task(run_training(client_id, config))
                manager.training_tasks[client_id] = task

            elif command.command == CommandType.PAUSE:
                if client_id in manager.trainers:
                    manager.trainers[client_id].pause()

            elif command.command == CommandType.RESUME:
                if client_id in manager.trainers:
                    manager.trainers[client_id].resume()

            elif command.command == CommandType.SET_SPEED:
                if client_id in manager.trainers and command.speed:
                    manager.trainers[client_id].set_speed(command.speed)

            elif command.command == CommandType.STOP:
                if client_id in manager.trainers:
                    manager.trainers[client_id].stop()
                if client_id in manager.training_tasks:
                    manager.training_tasks[client_id].cancel()

    except WebSocketDisconnect:
        manager.disconnect(client_id)
