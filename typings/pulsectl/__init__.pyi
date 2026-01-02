from types import TracebackType

from modal_dictation_exploration.types.pulsectl import (
    PulseSampleSpecDict,
    PulseSourceInfoDict,
)


class EnumValue:
    name: str


class PulseStateEnum:
    idle: EnumValue
    invalid: EnumValue
    running: EnumValue
    suspended: EnumValue


class PulseError(Exception): ...


class Pulse:
    def __init__(self, client_name: str) -> None: ...
    def __enter__(self) -> "Pulse": ...
    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None: ...
    def source_list(self) -> list[PulseSourceInfoDict]: ...
    def close(self) -> None: ...