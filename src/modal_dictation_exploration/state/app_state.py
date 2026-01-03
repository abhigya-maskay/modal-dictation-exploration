from dataclasses import dataclass

from modal_dictation_exploration.state.async_behavior_subject import (
    AsyncBehaviorSubject,
)


@dataclass
class AppState:
    selected_device: AsyncBehaviorSubject[int | None]
