use std::{
    collections::BTreeMap,
    io::{self, Read, Write},
    os::unix::net::UnixStream as StdUnixStream,
    sync::{Arc, Mutex},
    time::Duration,
};

#[cfg(target_os = "macos")]
use std::fs::File;
#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;

use clipmill_contracts::proto::{
    shm::v1::{BufferDescriptor, DataType, MapAcknowledgement, MapRequest, TransportType},
    time::v1::Timebase,
};
use prost::Message;
use sha2::{Digest, Sha256};
use thiserror::Error;

const MAX_FRAME_BYTES: usize = 4 * 1024 * 1024;
const SOCKET_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, Default)]
pub(crate) struct ShmBroker {
    entries: Arc<Mutex<BTreeMap<String, SharedEntry>>>,
}

impl ShmBroker {
    pub(crate) fn create(
        &self,
        lease_id: &str,
        bytes: &[u8],
    ) -> Result<BufferDescriptor, ShmError> {
        if bytes.is_empty() {
            return Err(ShmError::Invalid("shared-memory payload is empty"));
        }
        let token = random_token()?;
        let digest = hex::encode(Sha256::digest(bytes));
        let (backing, shm_name, transport_type) = create_backing(bytes)?;
        let descriptor = BufferDescriptor {
            shm_name,
            shape: vec![u64::try_from(bytes.len()).map_err(|_| ShmError::Overflow)?],
            dtype: DataType::U8 as i32,
            colorspace: String::new(),
            timebase: Some(Timebase {
                num: 1,
                den: 90_000,
            }),
            byte_len: u64::try_from(bytes.len()).map_err(|_| ShmError::Overflow)?,
            sha256: digest,
            lease_id: lease_id.to_owned(),
            transport_type: transport_type as i32,
            handle_token: token.clone(),
        };
        validate_descriptor(&descriptor)?;
        let mut entries = self.entries.lock().map_err(|_| ShmError::Stopped)?;
        entries.insert(
            token,
            SharedEntry {
                descriptor: descriptor.clone(),
                _backing: backing,
            },
        );
        Ok(descriptor)
    }

    pub(crate) fn revoke_lease(&self, lease_id: &str) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.retain(|_, entry| entry.descriptor.lease_id != lease_id);
        }
    }

    #[cfg(test)]
    fn outstanding(&self) -> usize {
        self.entries.lock().map_or(0, |entries| entries.len())
    }

    fn take(&self, request: &MapRequest) -> Result<SharedEntry, ShmError> {
        if request.lease_id.parse::<clipmill_core::LeaseId>().is_err()
            || request.handle_token.is_empty()
        {
            return Err(ShmError::Invalid("invalid shared-memory map request"));
        }
        let mut entries = self.entries.lock().map_err(|_| ShmError::Stopped)?;
        let entry = entries
            .remove(&request.handle_token)
            .ok_or(ShmError::UnknownHandle)?;
        if entry.descriptor.lease_id != request.lease_id {
            return Err(ShmError::LeaseMismatch);
        }
        Ok(entry)
    }
}

#[derive(Debug)]
struct SharedEntry {
    descriptor: BufferDescriptor,
    _backing: SharedBacking,
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct SharedBacking {
    memfd: memfd::Memfd,
}

#[cfg(target_os = "macos")]
#[derive(Debug)]
struct SharedBacking {
    _file: File,
    name: String,
}

#[cfg(target_os = "macos")]
impl Drop for SharedBacking {
    fn drop(&mut self) {
        tracing::debug!(operation = "unlink", "releasing POSIX shared memory");
        let _result = nix::sys::mman::shm_unlink(self.name.as_str());
    }
}

pub(crate) fn handle_shm_connection(
    mut stream: StdUnixStream,
    broker: &ShmBroker,
) -> Result<(), ShmError> {
    // Tokio sockets retain O_NONBLOCK when converted into std sockets. This
    // handler runs on a blocking thread and uses explicit socket deadlines.
    stream.set_nonblocking(false).map_err(ShmError::Io)?;
    stream
        .set_read_timeout(Some(SOCKET_TIMEOUT))
        .map_err(ShmError::Io)?;
    stream
        .set_write_timeout(Some(SOCKET_TIMEOUT))
        .map_err(ShmError::Io)?;
    let request = read_message::<MapRequest>(&mut stream)?;
    let entry = broker.take(&request)?;
    send_descriptor(&mut stream, &entry)?;
    let acknowledgement = read_message::<MapAcknowledgement>(&mut stream)?;
    if acknowledgement.lease_id != entry.descriptor.lease_id
        || acknowledgement.handle_token != entry.descriptor.handle_token
        || !acknowledgement.mapped
    {
        return Err(ShmError::Invalid(
            "shared-memory acknowledgement did not match the one-use handle",
        ));
    }
    // Dropping the taken entry closes the Linux memfd and unlinks the macOS
    // POSIX object. The worker retains only its read-only mapping.
    drop(entry);
    Ok(())
}

fn validate_descriptor(descriptor: &BufferDescriptor) -> Result<(), ShmError> {
    let element_bytes = match descriptor.dtype() {
        DataType::U8 => 1_u64,
        DataType::I16 | DataType::F16 => 2,
        DataType::I32 | DataType::F32 => 4,
        DataType::Unspecified => return Err(ShmError::Invalid("shared-memory dtype is missing")),
    };
    let elements = descriptor
        .shape
        .iter()
        .try_fold(1_u64, |product, dimension| {
            product.checked_mul(*dimension).ok_or(ShmError::Overflow)
        })?;
    if elements
        .checked_mul(element_bytes)
        .ok_or(ShmError::Overflow)?
        != descriptor.byte_len
    {
        return Err(ShmError::Invalid(
            "shared-memory shape does not match byte length",
        ));
    }
    if descriptor.timebase.as_ref().is_none_or(|timebase| {
        timebase.num <= 0 || timebase.den <= 0 || timebase.den > 1_000_000_000
    }) {
        return Err(ShmError::Invalid("shared-memory timebase is invalid"));
    }
    if descriptor.sha256.len() != 64
        || descriptor
            .sha256
            .bytes()
            .any(|byte| !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte))
    {
        return Err(ShmError::Invalid("shared-memory digest is invalid"));
    }
    Ok(())
}

fn random_token() -> Result<String, ShmError> {
    let mut random = [0_u8; 32];
    getrandom::fill(&mut random).map_err(|error| ShmError::Random(error.to_string()))?;
    Ok(format!("shm_{}", hex::encode(random)))
}

#[cfg(target_os = "linux")]
fn create_backing(bytes: &[u8]) -> Result<(SharedBacking, String, TransportType), ShmError> {
    use memfd::{FileSeal, MemfdOptions};

    let memfd = MemfdOptions::new()
        .allow_sealing(true)
        .create("clipmill-arrow")
        .map_err(|error| ShmError::Platform(error.to_string()))?;
    memfd
        .as_file()
        .set_len(u64::try_from(bytes.len()).map_err(|_| ShmError::Overflow)?)
        .map_err(ShmError::Io)?;
    let mut file = memfd.as_file();
    file.write_all(bytes).map_err(ShmError::Io)?;
    file.sync_all().map_err(ShmError::Io)?;
    let seals = [
        FileSeal::SealShrink,
        FileSeal::SealGrow,
        FileSeal::SealWrite,
        FileSeal::SealSeal,
    ];
    memfd
        .add_seals(&seals)
        .map_err(|error| ShmError::Platform(error.to_string()))?;
    Ok((
        SharedBacking { memfd },
        String::new(),
        TransportType::ScmRightsMemfd,
    ))
}

#[cfg(target_os = "macos")]
fn create_backing(bytes: &[u8]) -> Result<(SharedBacking, String, TransportType), ShmError> {
    use nix::{
        fcntl::OFlag,
        sys::{mman::shm_open, stat::Mode},
    };

    // Darwin limits POSIX shm names to 31 bytes including the leading slash.
    let name = format!("/cm_{}", ulid::Ulid::new());
    let owned = shm_open(
        name.as_str(),
        OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_RDWR,
        // The creator can populate its already-open O_RDWR descriptor, while
        // every subsequent open is constrained to read-only access.
        Mode::from_bits_truncate(0o400),
    )
    .map_err(|error| ShmError::Platform(error.to_string()))?;
    let populate = || -> Result<File, ShmError> {
        let file = File::from(owned);
        file.set_len(u64::try_from(bytes.len()).map_err(|_| ShmError::Overflow)?)
            .map_err(|error| ShmError::Platform(format!("POSIX shm truncate failed: {error}")))?;
        // Darwin POSIX shared-memory descriptors reject write(2) with ENXIO
        // and must be populated through a shared writable mapping. `mmap-io`
        // owns the unsafe boundary, preserving the workspace unsafe-code ban.
        let mapping =
            mmap_io::MemoryMappedFile::from_file(file, mmap_io::MmapMode::ReadWrite, name.as_str())
                .map_err(|error| {
                    ShmError::Platform(format!("POSIX shm mapping failed: {error}"))
                })?;
        mapping
            .update_region(0, bytes)
            .map_err(|error| ShmError::Platform(format!("POSIX shm copy failed: {error}")))?;
        mapping
            .flush()
            .map_err(|error| ShmError::Platform(format!("POSIX shm flush failed: {error}")))?;
        mapping.unmap().map_err(|_| {
            ShmError::Platform("POSIX shm mapping still had an active owner".to_owned())
        })
    };
    let file = match populate() {
        Ok(file) => file,
        Err(error) => {
            let _result = nix::sys::mman::shm_unlink(name.as_str());
            return Err(error);
        }
    };
    tracing::debug!(operation = "create", "created POSIX shared memory");
    Ok((
        SharedBacking {
            _file: file,
            name: name.clone(),
        },
        name,
        TransportType::PosixShm,
    ))
}

#[cfg(target_os = "linux")]
fn send_descriptor(stream: &mut StdUnixStream, entry: &SharedEntry) -> Result<(), ShmError> {
    use std::io::IoSlice;

    use nix::sys::socket::{ControlMessage, MsgFlags, sendmsg};

    let bytes = entry.descriptor.encode_length_delimited_to_vec();
    let descriptor = [entry._backing.memfd.as_file().as_raw_fd()];
    let iov = [IoSlice::new(&bytes)];
    let sent = sendmsg::<()>(
        stream.as_raw_fd(),
        &iov,
        &[ControlMessage::ScmRights(&descriptor)],
        MsgFlags::empty(),
        None,
    )
    .map_err(|error| ShmError::Platform(error.to_string()))?;
    if sent == 0 {
        return Err(ShmError::Invalid(
            "shared-memory descriptor write was empty",
        ));
    }
    if sent < bytes.len() {
        stream.write_all(&bytes[sent..]).map_err(ShmError::Io)?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn send_descriptor(stream: &mut StdUnixStream, entry: &SharedEntry) -> Result<(), ShmError> {
    let bytes = entry.descriptor.encode_length_delimited_to_vec();
    stream.write_all(&bytes).map_err(ShmError::Io)
}

fn read_message<M: Message + Default>(stream: &mut StdUnixStream) -> Result<M, ShmError> {
    let length = read_varint(stream)?;
    let length = usize::try_from(length).map_err(|_| ShmError::Overflow)?;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(ShmError::Invalid("invalid shared-memory frame length"));
    }
    let mut bytes = vec![0_u8; length];
    stream.read_exact(&mut bytes).map_err(ShmError::Io)?;
    M::decode(bytes.as_slice()).map_err(|error| ShmError::Decode(error.to_string()))
}

fn read_varint(stream: &mut StdUnixStream) -> Result<u64, ShmError> {
    let mut value = 0_u64;
    for index in 0..10 {
        let mut byte = [0_u8; 1];
        stream.read_exact(&mut byte).map_err(ShmError::Io)?;
        if index == 9 && byte[0] > 1 {
            return Err(ShmError::Invalid("malformed frame varint"));
        }
        value |= u64::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(ShmError::Invalid("malformed frame varint"))
}

#[derive(Debug, Error)]
pub(crate) enum ShmError {
    #[error("shared-memory protocol decode failed: {0}")]
    Decode(String),
    #[error("shared-memory I/O failed: {0}")]
    Io(#[source] io::Error),
    #[error("shared-memory request is invalid: {0}")]
    Invalid(&'static str),
    #[error("shared-memory handle belongs to another lease")]
    LeaseMismatch,
    #[error("shared-memory length overflow")]
    Overflow,
    #[error("shared-memory platform operation failed: {0}")]
    Platform(String),
    #[error("cannot generate a shared-memory handle: {0}")]
    Random(String),
    #[error("shared-memory broker stopped")]
    Stopped,
    #[error("shared-memory handle is unknown, expired, or already used")]
    UnknownHandle,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use clipmill_contracts::proto::shm::v1::MapRequest;

    use super::{ShmBroker, ShmError};

    #[test]
    fn descriptors_are_bound_to_one_use_handles_and_leases() {
        let broker = ShmBroker::default();
        let lease = clipmill_core::LeaseId::new().to_string();
        let descriptor = broker.create(&lease, b"arrow").expect("create descriptor");
        assert_eq!(broker.outstanding(), 1);
        #[cfg(target_os = "macos")]
        {
            let reopened = nix::sys::mman::shm_open(
                descriptor.shm_name.as_str(),
                nix::fcntl::OFlag::O_RDONLY,
                nix::sys::stat::Mode::empty(),
            )
            .expect("reopen POSIX shared memory read-only");
            drop(reopened);
        }
        let request = MapRequest {
            lease_id: lease.clone(),
            handle_token: descriptor.handle_token.clone(),
        };
        let entry = broker.take(&request).expect("take one-use handle");
        assert_eq!(broker.outstanding(), 0);
        assert!(matches!(
            broker.take(&request),
            Err(ShmError::UnknownHandle)
        ));
        drop(entry);

        let descriptor = broker.create(&lease, b"again").expect("second descriptor");
        assert_eq!(broker.outstanding(), 1);
        broker.revoke_lease(&lease);
        assert_eq!(broker.outstanding(), 0);
        assert_eq!(descriptor.byte_len, 5);
    }
}
