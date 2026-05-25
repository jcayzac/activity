#![warn(clippy::all)]
#![forbid(unsafe_code)]

#[allow(clippy::all, non_camel_case_types)]
pub mod biome {
    include!(concat!(env!("OUT_DIR"), "/biome.rs"));
}

pub mod segb;

use prost::Message as _;

const COCOA_EPOCH_OFFSET_S: f64 = 978_307_200.0;

#[inline]
fn cocoa_to_ms(cocoa: f64) -> i64 {
    ((COCOA_EPOCH_OFFSET_S + cocoa) * 1000.0) as i64
}

// ---------------------------------------------------------------------------
// InFocus parsing
// ---------------------------------------------------------------------------

/// A single app-focus-gained event extracted from an InFocusRecord payload.
pub struct InFocusEvent {
    pub time_ms: i64,
    pub bundle_id: String,
}

/// Parses an App.InFocus SEGB slot (proto bytes + zero padding + 4-byte CRC).
/// Returns `None` if the record is not a gained-focus event (field 3 != 1)
/// or required fields are missing.
///
/// We scan forward through proto fields to find exactly where the proto content
/// ends (at the first invalid/zero tag byte), then pass only those bytes to
/// prost::decode. This avoids both the zero-padding and the trailing CRC.
pub fn parse_infocus_record(slot: &[u8]) -> Option<InFocusEvent> {
    use prost::encoding::{WireType, decode_key, decode_varint};

    // Declared wire types for InFocusRecord fields (from biome_infocus.proto).
    // Used to detect when Apple reuses a field number with a different wire type
    // (observed: field 9 appears as LEN for ip_address, then again as Varint).
    // When wire type doesn't match the declaration, we stop — prost would reject it.
    fn expected_wire(field: u32) -> Option<WireType> {
        match field {
            2 | 3 | 13 => Some(WireType::Varint),
            4 => Some(WireType::SixtyFourBit),
            6 | 9 | 10 => Some(WireType::LengthDelimited),
            11 | 12 => Some(WireType::Varint),
            _ => None, // unknown field — any wire type accepted
        }
    }

    // Find the end of valid proto content by scanning fields.
    let mut cursor = slot;
    loop {
        let before = cursor;
        let (field, wire) = match decode_key(&mut cursor) {
            Err(_) => {
                cursor = before;
                break;
            }
            Ok(pair) => pair,
        };
        // Stop if wire type conflicts with the declared schema for this field.
        if let Some(expected) = expected_wire(field)
            && wire != expected
        {
            cursor = before;
            break;
        }
        match wire {
            WireType::Varint => {
                if decode_varint(&mut cursor).is_err() {
                    cursor = before;
                    break;
                }
            }
            WireType::SixtyFourBit => {
                if cursor.len() < 8 {
                    cursor = before;
                    break;
                }
                cursor = &cursor[8..];
            }
            WireType::LengthDelimited => match decode_varint(&mut cursor) {
                Err(_) => {
                    cursor = before;
                    break;
                }
                Ok(l) => {
                    let l = l as usize;
                    if cursor.len() < l {
                        cursor = before;
                        break;
                    }
                    cursor = &cursor[l..];
                }
            },
            WireType::ThirtyTwoBit => {
                if cursor.len() < 4 {
                    cursor = before;
                    break;
                }
                cursor = &cursor[4..];
            }
            _ => {
                cursor = before;
                break;
            }
        }
    }
    let proto_end = slot.len() - cursor.len();
    let record = biome::InFocusRecord::decode(&slot[..proto_end]).ok()?;
    if record.focus_gained != 1 {
        return None;
    }
    let ts = record.cocoa_ts;
    if ts == 0.0 {
        return None;
    }
    let bundle_id = String::from_utf8(record.bundle_id)
        .ok()
        .filter(|s| !s.is_empty())?;
    Some(InFocusEvent {
        time_ms: cocoa_to_ms(ts),
        bundle_id,
    })
}

// ---------------------------------------------------------------------------
// WiFi scanning
// ---------------------------------------------------------------------------

/// A WiFi connection session extracted from a Biome WiFi file.
pub struct WifiConnectionEvent {
    pub first_ms: i64,
    pub last_ms: i64,
    pub ssid: String,
}

const WIFI_MARKER: &[u8] = b"/wifi/connection";
const WIFI_SCAN_WINDOW: usize = 512;

/// Parses a WiFi session from the bytes immediately following a `/wifi/connection`
/// marker. The bytes are mid-stream protobuf (not a self-contained message), so
/// we use a tolerant field-by-field reader that skips unknown fields.
///
/// Observed wire structure (from hex dump):
///   field 2 (LEN): inner submessage (skip — not the timestamps)
///   field 2 (I64): cocoa_start (double)
///   field 3 (I64): cocoa_end   (double)
///   field 4 (LEN): SSID wrapper
///     field 3 (LEN): SSID string
///
/// Note: field 2 appears twice (once LEN, once I64). We handle both.
fn parse_wifi_record(buf: &[u8]) -> Option<WifiConnectionEvent> {
    use prost::encoding::{WireType, decode_key, decode_varint};

    fn skip_len(c: &mut &[u8]) -> bool {
        let mut tmp = *c;
        let Ok(l) = decode_varint(&mut tmp) else {
            return false;
        };
        let l = l as usize;
        if tmp.len() < l {
            return false;
        }
        *c = &tmp[l..];
        true
    }

    fn read_i64_as_f64(c: &mut &[u8]) -> Option<f64> {
        if c.len() < 8 {
            return None;
        }
        let v = f64::from_le_bytes(c[..8].try_into().ok()?);
        *c = &c[8..];
        Some(v)
    }

    fn read_len_bytes<'a>(c: &mut &'a [u8]) -> Option<&'a [u8]> {
        let mut tmp = *c;
        let Ok(l) = decode_varint(&mut tmp) else {
            return None;
        };
        let l = l as usize;
        if tmp.len() < l {
            return None;
        }
        let bytes = &tmp[..l];
        *c = &tmp[l..];
        Some(bytes)
    }

    let mut cursor = buf;
    let mut cocoa_start: Option<f64> = None;
    let mut cocoa_end: Option<f64> = None;
    let mut ssid: Option<String> = None;

    while !(cursor.is_empty() || cocoa_start.is_some() && cocoa_end.is_some() && ssid.is_some()) {
        let Ok((field, wire)) = decode_key(&mut cursor) else {
            break;
        };
        match (field, wire) {
            (2, WireType::SixtyFourBit) => {
                cocoa_start = read_i64_as_f64(&mut cursor);
            }
            (3, WireType::SixtyFourBit) => {
                cocoa_end = read_i64_as_f64(&mut cursor);
            }
            (4, WireType::LengthDelimited) => {
                // SSID wrapper: contains field 3 (LEN) = SSID string
                if let Some(inner) = read_len_bytes(&mut cursor) {
                    let mut ic = inner;
                    while !ic.is_empty() {
                        let Ok((f, w)) = decode_key(&mut ic) else {
                            break;
                        };
                        match (f, w) {
                            (3, WireType::LengthDelimited) => {
                                if let Some(sb) = read_len_bytes(&mut ic) {
                                    ssid = std::str::from_utf8(sb).ok().map(|s| s.to_owned());
                                }
                            }
                            (_, WireType::LengthDelimited) => {
                                skip_len(&mut ic);
                            }
                            (_, WireType::SixtyFourBit) => {
                                if ic.len() < 8 {
                                    break;
                                }
                                ic = &ic[8..];
                            }
                            (_, WireType::ThirtyTwoBit) => {
                                if ic.len() < 4 {
                                    break;
                                }
                                ic = &ic[4..];
                            }
                            (_, WireType::Varint) => {
                                let mut b = ic;
                                let _ = decode_varint(&mut b);
                                ic = b;
                            }
                            _ => break,
                        }
                    }
                }
            }
            (_, WireType::LengthDelimited) => {
                skip_len(&mut cursor);
            }
            (_, WireType::SixtyFourBit) => {
                if cursor.len() < 8 {
                    break;
                }
                cursor = &cursor[8..];
            }
            (_, WireType::ThirtyTwoBit) => {
                if cursor.len() < 4 {
                    break;
                }
                cursor = &cursor[4..];
            }
            (_, WireType::Varint) => {
                let mut b = cursor;
                let _ = decode_varint(&mut b);
                cursor = b;
            }
            _ => break,
        }
    }

    let ssid = ssid.filter(|s| !s.is_empty())?;
    let first_ms = cocoa_to_ms(cocoa_start?);
    let last_ms = cocoa_to_ms(cocoa_end?);
    if first_ms >= last_ms {
        return None;
    }
    Some(WifiConnectionEvent {
        first_ms,
        last_ms,
        ssid,
    })
}

/// Finds all WiFi connection records in a raw file buffer by scanning for the
/// `/wifi/connection` marker. WiFi files use marker-scan, not SEGB framing.
pub fn scan_wifi_records(file_bytes: &[u8]) -> Vec<WifiConnectionEvent> {
    let mut events = Vec::new();
    for offset in memchr::memmem::find_iter(file_bytes, WIFI_MARKER) {
        let start = offset + WIFI_MARKER.len();
        let end = (start + WIFI_SCAN_WINDOW).min(file_bytes.len());
        if start >= end {
            continue;
        }
        if let Some(ev) = parse_wifi_record(&file_bytes[start..end]) {
            events.push(ev);
        }
    }
    events
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{COCOA_EPOCH_OFFSET_S, biome, parse_infocus_record, scan_wifi_records};
    use prost::Message as _;

    fn encode_infocus(
        record_type: i32,
        focus_gained: i32,
        cocoa_ts: f64,
        bundle_id: &[u8],
    ) -> Vec<u8> {
        biome::InFocusRecord {
            record_type,
            focus_gained,
            cocoa_ts,
            bundle_id: bundle_id.to_vec(),
            ..Default::default()
        }
        .encode_to_vec()
    }

    #[test]
    fn parse_infocus_valid() {
        let bundle = b"com.apple.Safari";
        // cocoa_ts 0.0 is the epoch itself (2001-01-01), but the implementation
        // treats 0.0 as "missing". Use a real timestamp instead.
        let cocoa_ts = 750_000_000.0_f64; // some time in 2024
        let payload = encode_infocus(1, 1, cocoa_ts, bundle);
        let event = parse_infocus_record(&payload).expect("should parse");
        assert_eq!(event.bundle_id, "com.apple.Safari");
        let expected_ms = ((COCOA_EPOCH_OFFSET_S + cocoa_ts) * 1000.0) as i64;
        assert_eq!(event.time_ms, expected_ms);
    }

    #[test]
    fn parse_infocus_focus_gained_zero_rejected() {
        let payload = encode_infocus(1, 0, 750_000_000.0, b"com.apple.Safari");
        assert!(parse_infocus_record(&payload).is_none());
    }

    #[test]
    fn parse_infocus_zero_ts_rejected() {
        let payload = encode_infocus(1, 1, 0.0, b"com.apple.Safari");
        assert!(parse_infocus_record(&payload).is_none());
    }

    #[test]
    fn parse_infocus_empty_bundle_rejected() {
        let payload = encode_infocus(1, 1, 750_000_000.0, b"");
        assert!(parse_infocus_record(&payload).is_none());
    }

    #[test]
    fn segb_record_count_from_real_files() {
        let base = format!(
            "{}/Library/Biome/streams/restricted/App.InFocus/local",
            std::env::var("HOME").unwrap_or_default()
        );
        let base = camino::Utf8Path::new(&base);
        if !base.exists() {
            return;
        }

        let mut total = 0usize;
        let mut parsed_ok = 0usize;
        let mut files: Vec<_> = std::fs::read_dir(base.as_std_path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .is_some_and(|n| n.chars().all(|c| c.is_ascii_digit()))
            })
            .map(|e| camino::Utf8PathBuf::from_path_buf(e.path()).unwrap())
            .collect();
        files.sort();
        for path in &files {
            let iter = crate::segb::iter_records(path).unwrap();
            for payload in iter {
                total += 1;
                if parse_infocus_record(&payload).is_some() {
                    parsed_ok += 1;
                }
            }
        }
        eprintln!("SEGB total records: {total}, parsed gained-focus: {parsed_ok}");

        assert!(total > 30_000, "expected >30k SEGB records, got {total}");
        assert!(
            parsed_ok > 10_000,
            "expected >10k gained-focus events, got {parsed_ok}"
        );
    }

    #[test]
    fn parse_infocus_with_extra_fields_and_crc() {
        // Real SEGB slot layout: proto bytes + zero padding + 4-byte CRC.
        // Uses the actual bytes from an observed App.InFocus record.
        let slot: &[u8] = &[
            0x10, 0x01, // field 2 = 1
            0x18, 0x01, // field 3 = 1 (gained)
            0x21, 0xb7, 0x09, 0xa7, 0x5d, 0x63, 0xcf, 0xc7, 0x41, // field 4 double
            0x32, 0x11, // field 6 LEN len=17
            b'c', b'o', b'm', b'.', b'b', b'r', b'a', b'v', b'e', b'.', b'B', b'r', b'o', b'w',
            b's', b'e', b'r', 0x4a, 0x0c, // field 9 (ip_address) LEN len=12
            b'1', b'4', b'7', b'.', b'1', b'.', b'0', b'.', b'3', b'5', b'9', b'1', 0x58,
            0x01, // field 11 (bool) = true
            0x60, 0x01, // field 12 (bool) = true
            0x68, 0x00, // field 13 (uint32) = 0
            0x00, 0x00, 0x00, 0x00, // zero padding
            0x26, 0x7e, 0x9b, 0x45, // 4-byte CRC (stripped before decode)
        ];
        let result = parse_infocus_record(slot);
        assert!(
            result.is_some(),
            "should parse real slot with extra fields and CRC"
        );
        let ev = result.unwrap();
        assert_eq!(ev.bundle_id, "com.brave.Browser");
    }

    #[test]
    fn parse_infocus_zoom_slot_with_endgroup_byte() {
        // Real App.InFocus slot ending with 0xc4 (EndGroup wire type / truncated varint).
        // This slot is for us.zoom.xos at 2026-05-12 15:01:16 and must parse successfully.
        let slot: &[u8] = &[
            0x10, 0x01, 0x18, 0x01, // field 2=1, field 3=1 (gained)
            0x21, 0x1a, 0x34, 0x7e, 0xd6, 0x7c, 0xd9, 0xc7, 0x41, // field 4 double
            0x32, 0x0b, // field 6 LEN=11
            b'u', b's', b'.', b'z', b'o', b'o', b'm', b'.', b'x', b'o', b's', 0x4a,
            0x0d, // field 9 LEN=13
            b'7', b'.', b'0', b'.', b'0', b' ', b'(', b'7', b'7', b'5', b'9', b'3', b')', 0x52,
            0x0b, // field 10 LEN=11
            b'7', b'.', b'0', b'.', b'0', b'.', b'7', b'7', b'5', b'9', b'3', 0x58,
            0x01, // field 11 = 1
            0x60, 0x01, // field 12 = 1
            0x68, 0x00, // field 13 = 0
            0x48, 0xd1, 0x44, // field 9 varint = 8785 (extra unknown)
            0xc4, // truncated varint tag (EndGroup-like) — must stop scan here
        ];
        let result = parse_infocus_record(slot);
        assert!(
            result.is_some(),
            "should parse zoom slot with truncated trailing byte"
        );
        let ev = result.unwrap();
        assert_eq!(ev.bundle_id, "us.zoom.xos");
    }

    #[test]
    fn scan_wifi_records_empty_buffer() {
        let events = scan_wifi_records(&[]);
        assert!(events.is_empty());
    }

    #[test]
    fn scan_wifi_records_no_marker() {
        let events = scan_wifi_records(b"hello world no marker here");
        assert!(events.is_empty());
    }
}
