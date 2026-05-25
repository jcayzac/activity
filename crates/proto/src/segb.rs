#![forbid(unsafe_code)]
//! Streaming reader for the SEGB binary container format used by macOS Biome.
//!
//! A SEGB file consists of:
//! - A variable-length header (ends at a delimiter sentinel where the 4 bytes
//!   after the delimiter are all `0x00` or `0xff` padding)
//! - A sequence of records, each preceded by the 4-byte delimiter
//!   `[0x0a, 0x00, 0x00, 0x00]`

use thiserror::Error;

const DELIMITER: &[u8] = &[0x0a, 0x00, 0x00, 0x00];

/// Errors returned by the SEGB reader.
#[derive(Debug, Error)]
pub enum SegbError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// An iterator over raw record payloads in a SEGB file.
pub struct RecordIter {
    data: Vec<u8>,
    /// Pre-computed `(start, end)` byte ranges for each record payload.
    ranges: Vec<(usize, usize)>,
    index: usize,
}

impl RecordIter {
    fn new(data: Vec<u8>) -> Self {
        let ranges = collect_record_ranges(&data);
        RecordIter {
            data,
            ranges,
            index: 0,
        }
    }
}

impl Iterator for RecordIter {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let (start, end) = *self.ranges.get(self.index)?;
        self.index += 1;
        Some(self.data[start..end].to_vec())
    }
}

/// Scans the byte slice and returns `(start, end)` ranges for each record payload.
fn collect_record_ranges(data: &[u8]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut header_consumed = false;
    let dlen = DELIMITER.len(); // 4

    // Walk every occurrence of the delimiter.
    let mut search_start = 0usize;
    loop {
        let Some(rel) = memchr::memmem::find(&data[search_start..], DELIMITER) else {
            break;
        };
        let delim_pos = search_start + rel;
        let payload_start = delim_pos + dlen;

        if !header_consumed {
            // Check the byte immediately after the delimiter.
            // If the buffer is exhausted here or the byte is 0x00/0xff, this
            // is the header sentinel.
            let sentinel = data.get(payload_start).copied();
            if sentinel.is_none_or(|b| b == 0x00 || b == 0xff) {
                header_consumed = true;
                search_start = payload_start;
                continue;
            }
            // Otherwise this delimiter is inside the header — skip it.
            search_start = payload_start;
            continue;
        }

        // Find where this record ends: at the next delimiter (or EOF).
        let payload_end = memchr::memmem::find(&data[payload_start..], DELIMITER)
            .map(|r| payload_start + r)
            .unwrap_or(data.len());

        if payload_start < payload_end {
            ranges.push((payload_start, payload_end));
        }

        search_start = payload_end;
        if search_start >= data.len() {
            break;
        }
    }

    ranges
}

/// Opens a SEGB file, reads it fully into memory, and returns an iterator over
/// its raw record payloads.
///
/// The file is read all at once so that a concurrent writer (e.g. the Biome
/// daemon) cannot alter the bytes seen by the iterator after the read completes.
pub fn iter_records(path: &camino::Utf8Path) -> Result<RecordIter, SegbError> {
    let data = std::fs::read(path.as_std_path())?;
    Ok(RecordIter::new(data))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{DELIMITER, collect_record_ranges};

    /// Builds a minimal SEGB buffer with the given payloads.
    fn build_segb(payloads: &[&[u8]]) -> Vec<u8> {
        let mut buf = Vec::new();
        // Header: a delimiter followed by 4 zero bytes (sentinel).
        buf.extend_from_slice(DELIMITER);
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        // Records.
        for payload in payloads {
            buf.extend_from_slice(DELIMITER);
            buf.extend_from_slice(payload);
        }
        buf
    }

    #[test]
    fn no_records() {
        let buf = build_segb(&[]);
        let ranges = collect_record_ranges(&buf);
        assert!(ranges.is_empty());
    }

    #[test]
    fn single_record() {
        let buf = build_segb(&[b"hello"]);
        let ranges = collect_record_ranges(&buf);
        assert_eq!(ranges.len(), 1);
        let (s, e) = ranges[0];
        assert_eq!(&buf[s..e], b"hello");
    }

    #[test]
    fn multiple_records() {
        let payloads: &[&[u8]] = &[b"first", b"second", b"third"];
        let buf = build_segb(payloads);
        let ranges = collect_record_ranges(&buf);
        assert_eq!(ranges.len(), 3);
        for (i, &(s, e)) in ranges.iter().enumerate() {
            assert_eq!(&buf[s..e], payloads[i]);
        }
    }

    #[test]
    fn record_containing_delimiter_bytes() {
        // A payload that happens to start with 0x0a (a common protobuf wire
        // byte) should not be misidentified as a delimiter.  The full 4-byte
        // sequence is required.
        let payload = &[0x0a, 0x01, 0x02, 0x03, 0x04];
        let buf = build_segb(&[payload]);
        let ranges = collect_record_ranges(&buf);
        assert_eq!(ranges.len(), 1);
        let (s, e) = ranges[0];
        assert_eq!(&buf[s..e], payload);
    }
}
