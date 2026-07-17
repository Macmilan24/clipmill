from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class Timebase(_message.Message):
    __slots__ = ("num", "den")
    NUM_FIELD_NUMBER: _ClassVar[int]
    DEN_FIELD_NUMBER: _ClassVar[int]
    num: int
    den: int
    def __init__(self, num: _Optional[int] = ..., den: _Optional[int] = ...) -> None: ...

class Ticks(_message.Message):
    __slots__ = ("ticks",)
    TICKS_FIELD_NUMBER: _ClassVar[int]
    ticks: int
    def __init__(self, ticks: _Optional[int] = ...) -> None: ...

class Interval(_message.Message):
    __slots__ = ("start_ticks", "end_ticks")
    START_TICKS_FIELD_NUMBER: _ClassVar[int]
    END_TICKS_FIELD_NUMBER: _ClassVar[int]
    start_ticks: int
    end_ticks: int
    def __init__(self, start_ticks: _Optional[int] = ..., end_ticks: _Optional[int] = ...) -> None: ...
