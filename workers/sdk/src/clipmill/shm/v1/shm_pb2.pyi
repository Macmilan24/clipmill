from clipmill.time.v1 import time_pb2 as _time_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class TransportType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    TRANSPORT_TYPE_UNSPECIFIED: _ClassVar[TransportType]
    TRANSPORT_TYPE_SCM_RIGHTS_MEMFD: _ClassVar[TransportType]
    TRANSPORT_TYPE_POSIX_SHM: _ClassVar[TransportType]

class DataType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DATA_TYPE_UNSPECIFIED: _ClassVar[DataType]
    DATA_TYPE_U8: _ClassVar[DataType]
    DATA_TYPE_I16: _ClassVar[DataType]
    DATA_TYPE_I32: _ClassVar[DataType]
    DATA_TYPE_F16: _ClassVar[DataType]
    DATA_TYPE_F32: _ClassVar[DataType]
TRANSPORT_TYPE_UNSPECIFIED: TransportType
TRANSPORT_TYPE_SCM_RIGHTS_MEMFD: TransportType
TRANSPORT_TYPE_POSIX_SHM: TransportType
DATA_TYPE_UNSPECIFIED: DataType
DATA_TYPE_U8: DataType
DATA_TYPE_I16: DataType
DATA_TYPE_I32: DataType
DATA_TYPE_F16: DataType
DATA_TYPE_F32: DataType

class BufferDescriptor(_message.Message):
    __slots__ = ("shm_name", "shape", "dtype", "colorspace", "timebase", "byte_len", "sha256", "lease_id", "transport_type", "handle_token")
    SHM_NAME_FIELD_NUMBER: _ClassVar[int]
    SHAPE_FIELD_NUMBER: _ClassVar[int]
    DTYPE_FIELD_NUMBER: _ClassVar[int]
    COLORSPACE_FIELD_NUMBER: _ClassVar[int]
    TIMEBASE_FIELD_NUMBER: _ClassVar[int]
    BYTE_LEN_FIELD_NUMBER: _ClassVar[int]
    SHA256_FIELD_NUMBER: _ClassVar[int]
    LEASE_ID_FIELD_NUMBER: _ClassVar[int]
    TRANSPORT_TYPE_FIELD_NUMBER: _ClassVar[int]
    HANDLE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    shm_name: str
    shape: _containers.RepeatedScalarFieldContainer[int]
    dtype: DataType
    colorspace: str
    timebase: _time_pb2.Timebase
    byte_len: int
    sha256: str
    lease_id: str
    transport_type: TransportType
    handle_token: str
    def __init__(self, shm_name: _Optional[str] = ..., shape: _Optional[_Iterable[int]] = ..., dtype: _Optional[_Union[DataType, str]] = ..., colorspace: _Optional[str] = ..., timebase: _Optional[_Union[_time_pb2.Timebase, _Mapping]] = ..., byte_len: _Optional[int] = ..., sha256: _Optional[str] = ..., lease_id: _Optional[str] = ..., transport_type: _Optional[_Union[TransportType, str]] = ..., handle_token: _Optional[str] = ...) -> None: ...

class MapRequest(_message.Message):
    __slots__ = ("lease_id", "handle_token")
    LEASE_ID_FIELD_NUMBER: _ClassVar[int]
    HANDLE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    lease_id: str
    handle_token: str
    def __init__(self, lease_id: _Optional[str] = ..., handle_token: _Optional[str] = ...) -> None: ...

class MapAcknowledgement(_message.Message):
    __slots__ = ("lease_id", "handle_token", "mapped", "detail")
    LEASE_ID_FIELD_NUMBER: _ClassVar[int]
    HANDLE_TOKEN_FIELD_NUMBER: _ClassVar[int]
    MAPPED_FIELD_NUMBER: _ClassVar[int]
    DETAIL_FIELD_NUMBER: _ClassVar[int]
    lease_id: str
    handle_token: str
    mapped: bool
    detail: str
    def __init__(self, lease_id: _Optional[str] = ..., handle_token: _Optional[str] = ..., mapped: _Optional[bool] = ..., detail: _Optional[str] = ...) -> None: ...
