use crate::types::HasTime;

/// Binary search for the last element whose `.time_ms()` is strictly less
/// than `t`. Returns `None` if no such element exists.
pub fn last_before<T: HasTime>(arr: &[T], t: i64) -> Option<&T> {
    if arr.is_empty() {
        return None;
    }
    let mut lo: usize = 0;
    let mut hi: usize = arr.len().saturating_sub(1);
    let mut result: Option<usize> = None;
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        if arr[mid].time_ms() < t {
            result = Some(mid);
            lo = mid + 1;
        } else {
            if mid == 0 {
                break;
            }
            hi = mid - 1;
        }
    }
    result.map(|i| &arr[i])
}

#[cfg(test)]
mod tests {
    use super::last_before;
    use crate::types::{RawEvent, RawEventKind};

    #[test]
    fn last_before_empty() {
        let arr: Vec<RawEvent> = vec![];
        assert!(last_before(&arr, 1000).is_none());
    }

    #[test]
    fn last_before_all_ge() {
        let arr = vec![
            RawEvent { time_ms: 100, kind: RawEventKind::Kbd },
            RawEvent { time_ms: 200, kind: RawEventKind::Kbd },
        ];
        assert!(last_before(&arr, 100).is_none());
        assert!(last_before(&arr, 50).is_none());
    }

    #[test]
    fn last_before_found() {
        let arr = vec![
            RawEvent { time_ms: 100, kind: RawEventKind::Kbd },
            RawEvent { time_ms: 200, kind: RawEventKind::Kbd },
            RawEvent { time_ms: 300, kind: RawEventKind::Kbd },
        ];
        let r = last_before(&arr, 250).unwrap();
        assert_eq!(r.time_ms, 200);
    }

    #[test]
    fn last_before_boundary_exact() {
        let arr = vec![
            RawEvent { time_ms: 100, kind: RawEventKind::Kbd },
            RawEvent { time_ms: 200, kind: RawEventKind::Kbd },
        ];
        let r = last_before(&arr, 200).unwrap();
        assert_eq!(r.time_ms, 100);
    }

    #[test]
    fn last_before_returns_last_element() {
        let arr = vec![
            RawEvent { time_ms: 100, kind: RawEventKind::Kbd },
            RawEvent { time_ms: 200, kind: RawEventKind::Kbd },
        ];
        let r = last_before(&arr, 999).unwrap();
        assert_eq!(r.time_ms, 200);
    }
}
