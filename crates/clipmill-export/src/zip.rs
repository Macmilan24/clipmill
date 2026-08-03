//! A ZIP writer that produces the same bytes twice.
//!
//! The archive is the promise that a user's work outlives this application, so
//! the format has to be one every operating system opens without being told
//! how — which means ZIP, and means the bytes have to be right rather than
//! nearly right.
//!
//! Written here rather than taken from a crate for one reason: **determinism**.
//! An archive of the same project must be the same file, so it can be hashed,
//! compared, and round-trip tested. Every general-purpose writer stamps the
//! current time into each entry, and several also record the host platform and
//! permission bits, all of which make two archives of identical content differ.
//! Fixing that from the outside means overriding most of what the library does.
//! So entries are stored (never deflated), timestamps are pinned to the epoch
//! the format itself starts at, and the caller controls the order.
//!
//! Storing rather than deflating costs size on the JSON this archive holds. It
//! buys a writer with no compression state to get wrong, and an archive whose
//! entries can be read back by seeking — and the size is bounded by the
//! documents a project actually contains, not by its media, which the archive
//! references rather than copies.
//!
//! Zip64 is not implemented, and the two limits that would need it are refused
//! with a reason rather than written as a file that some tools open and others
//! do not.

/// The DOS timestamp the format's own epoch: 1980-01-01 00:00:00.
///
/// Not "now". An archive that recorded when it was made would differ from an
/// archive of the same project made a second later, and the round-trip gate
/// could not compare bytes.
const DOS_DATE: u16 = 0x0021;
const DOS_TIME: u16 = 0x0000;

const LOCAL_HEADER: u32 = 0x0403_4b50;
const CENTRAL_HEADER: u32 = 0x0201_4b50;
const END_OF_CENTRAL_DIRECTORY: u32 = 0x0605_4b50;

/// Bit 11: the name is UTF-8. Set unconditionally, because a project name is
/// not necessarily ASCII and the alternative encoding is a code page nobody
/// can identify from the file.
const FLAG_UTF8: u16 = 0x0800;
const METHOD_STORE: u16 = 0;
/// 2.0, which is what "store, with a data descriptor absent" needs.
const VERSION_NEEDED: u16 = 20;

const MAX_ENTRIES: usize = u16::MAX as usize;
const MAX_BYTES: u64 = u32::MAX as u64;

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ZipError {
    #[error(
        "an archive of more than {MAX_ENTRIES} entries needs Zip64, which this writer does not produce"
    )]
    TooManyEntries,
    #[error("an archive over 4 GiB needs Zip64, which this writer does not produce")]
    TooLarge,
    #[error("`{name}` is not a name an archive entry may have")]
    BadName { name: String },
}

/// Builds one archive in memory.
///
/// In memory because the archive holds a project's documents rather than its
/// media — kilobytes to a few megabytes — and because a partially written file
/// on disk is a worse failure than a refused allocation.
#[derive(Debug, Default)]
pub struct ZipWriter {
    bytes: Vec<u8>,
    entries: Vec<Entry>,
}

#[derive(Debug)]
struct Entry {
    name: String,
    crc: u32,
    size: u32,
    offset: u32,
}

impl ZipWriter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one file. Names are forward-slash separated and relative, which is
    /// what the format requires and what stops an archive writing outside the
    /// directory it is extracted into.
    pub fn add(&mut self, name: &str, contents: &[u8]) -> Result<(), ZipError> {
        if name.is_empty()
            || name.starts_with('/')
            || name.contains('\\')
            || name.split('/').any(|part| part == "..")
        {
            return Err(ZipError::BadName {
                name: name.to_owned(),
            });
        }
        if self.entries.len() >= MAX_ENTRIES {
            return Err(ZipError::TooManyEntries);
        }
        let offset = u32::try_from(self.bytes.len()).map_err(|_| ZipError::TooLarge)?;
        let size = u32::try_from(contents.len()).map_err(|_| ZipError::TooLarge)?;
        let crc = crc32(contents);
        let name_bytes = name.as_bytes();
        let name_length = u16::try_from(name_bytes.len()).map_err(|_| ZipError::TooLarge)?;

        self.push_u32(LOCAL_HEADER);
        self.push_u16(VERSION_NEEDED);
        self.push_u16(FLAG_UTF8);
        self.push_u16(METHOD_STORE);
        self.push_u16(DOS_TIME);
        self.push_u16(DOS_DATE);
        self.push_u32(crc);
        self.push_u32(size);
        self.push_u32(size);
        self.push_u16(name_length);
        self.push_u16(0);
        self.bytes.extend_from_slice(name_bytes);
        self.bytes.extend_from_slice(contents);

        if self.bytes.len() as u64 > MAX_BYTES {
            return Err(ZipError::TooLarge);
        }
        self.entries.push(Entry {
            name: name.to_owned(),
            crc,
            size,
            offset,
        });
        Ok(())
    }

    /// Close the archive and hand back its bytes.
    pub fn finish(mut self) -> Result<Vec<u8>, ZipError> {
        let directory_offset = u32::try_from(self.bytes.len()).map_err(|_| ZipError::TooLarge)?;
        let entries = std::mem::take(&mut self.entries);
        for entry in &entries {
            let name_bytes = entry.name.as_bytes();
            let name_length = u16::try_from(name_bytes.len()).map_err(|_| ZipError::TooLarge)?;
            self.push_u32(CENTRAL_HEADER);
            // Version made by: 2.0, host 0 (MS-DOS). Host zero rather than the
            // platform doing the writing, so a macOS archive and a Linux
            // archive of the same project are the same file.
            self.push_u16(VERSION_NEEDED);
            self.push_u16(VERSION_NEEDED);
            self.push_u16(FLAG_UTF8);
            self.push_u16(METHOD_STORE);
            self.push_u16(DOS_TIME);
            self.push_u16(DOS_DATE);
            self.push_u32(entry.crc);
            self.push_u32(entry.size);
            self.push_u32(entry.size);
            self.push_u16(name_length);
            self.push_u16(0);
            self.push_u16(0);
            self.push_u16(0);
            self.push_u16(0);
            // External attributes zero: no permission bits, for the same
            // reason the host is zero.
            self.push_u32(0);
            self.push_u32(entry.offset);
            self.bytes.extend_from_slice(name_bytes);
        }
        let directory_size =
            u32::try_from(self.bytes.len()).map_err(|_| ZipError::TooLarge)? - directory_offset;
        let count = u16::try_from(entries.len()).map_err(|_| ZipError::TooManyEntries)?;

        self.push_u32(END_OF_CENTRAL_DIRECTORY);
        self.push_u16(0);
        self.push_u16(0);
        self.push_u16(count);
        self.push_u16(count);
        self.push_u32(directory_size);
        self.push_u32(directory_offset);
        self.push_u16(0);
        Ok(self.bytes)
    }

    fn push_u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn push_u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }
}

/// CRC-32, the IEEE polynomial in its reflected form, which is what ZIP wants.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::{ZipError, ZipWriter, crc32};

    fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = ZipWriter::new();
        for (name, contents) in entries {
            writer.add(name, contents).expect("the entry is addable");
        }
        writer.finish().expect("the archive closes")
    }

    #[test]
    fn the_crc_matches_the_published_check_value() {
        // The IEEE 802.3 check value for "123456789", which is how every CRC-32
        // implementation states which variant it is.
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn the_same_contents_produce_the_same_bytes() {
        let once = archive(&[("index.json", b"{}"), ("docs/a.json", b"[1,2]")]);
        let again = archive(&[("index.json", b"{}"), ("docs/a.json", b"[1,2]")]);
        assert_eq!(once, again);
    }

    #[test]
    fn it_starts_and_ends_where_the_format_says() {
        let bytes = archive(&[("a.txt", b"hello")]);
        assert_eq!(&bytes[..4], b"PK\x03\x04");
        assert_eq!(&bytes[bytes.len() - 22..bytes.len() - 18], b"PK\x05\x06");
    }

    #[test]
    fn the_directory_counts_and_locates_every_entry() {
        let bytes = archive(&[("a", b"1"), ("b", b"22"), ("c", b"333")]);
        let end = bytes.len() - 22;
        let count = u16::from_le_bytes([bytes[end + 10], bytes[end + 11]]);
        assert_eq!(count, 3);
        let size = u32::from_le_bytes(bytes[end + 12..end + 16].try_into().unwrap());
        let offset = u32::from_le_bytes(bytes[end + 16..end + 20].try_into().unwrap());
        assert_eq!(offset as usize + size as usize, end);
        assert_eq!(&bytes[offset as usize..offset as usize + 4], b"PK\x01\x02");
    }

    #[test]
    fn an_entry_that_would_escape_the_extraction_directory_is_refused() {
        let mut writer = ZipWriter::new();
        for name in ["/etc/passwd", "../secrets.json", "a\\b.json", ""] {
            assert!(
                matches!(writer.add(name, b"x"), Err(ZipError::BadName { .. })),
                "{name} was accepted"
            );
        }
        // A nested path that does not escape is fine.
        assert!(writer.add("docs/edit/a.json", b"x").is_ok());
    }

    #[test]
    fn nothing_records_when_the_archive_was_made() {
        let bytes = archive(&[("a.txt", b"hello")]);
        // Local header time and date, at offsets 10 and 12.
        assert_eq!(u16::from_le_bytes([bytes[10], bytes[11]]), 0);
        assert_eq!(u16::from_le_bytes([bytes[12], bytes[13]]), 0x0021);
    }

    #[test]
    fn an_empty_archive_is_still_a_valid_one() {
        let bytes = archive(&[]);
        assert_eq!(bytes.len(), 22);
        assert_eq!(&bytes[..4], b"PK\x05\x06");
    }
}
