from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class PingRequest(_message.Message):
    __slots__ = ("echo",)
    ECHO_FIELD_NUMBER: _ClassVar[int]
    echo: str
    def __init__(self, echo: _Optional[str] = ...) -> None: ...

class PingResponse(_message.Message):
    __slots__ = ("echo", "daemon_version")
    ECHO_FIELD_NUMBER: _ClassVar[int]
    DAEMON_VERSION_FIELD_NUMBER: _ClassVar[int]
    echo: str
    daemon_version: str
    def __init__(self, echo: _Optional[str] = ..., daemon_version: _Optional[str] = ...) -> None: ...
