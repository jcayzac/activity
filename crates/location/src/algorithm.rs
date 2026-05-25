/// Orders two signals so the primary key (signal_a, signal_b) is stable.
/// "duet:" sorts before "subnet:"; among the same prefix, lex-smaller wins.
pub fn order_signals(x: &str, y: &str) -> [String; 2] {
    let x_duet = x.starts_with("duet:");
    let y_duet = y.starts_with("duet:");
    if x_duet && !y_duet {
        return [x.to_string(), y.to_string()];
    }
    if y_duet && !x_duet {
        return [y.to_string(), x.to_string()];
    }
    if x <= y {
        [x.to_string(), y.to_string()]
    } else {
        [y.to_string(), x.to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::order_signals;

    #[test]
    fn duet_before_subnet() {
        let [a, b] = order_signals("subnet:10.0.0.0/24", "duet:abc");
        assert_eq!(a, "duet:abc");
        assert_eq!(b, "subnet:10.0.0.0/24");
    }

    #[test]
    fn lex_within_same_prefix_duet() {
        let [a, b] = order_signals("duet:zzz", "duet:aaa");
        assert_eq!(a, "duet:aaa");
        assert_eq!(b, "duet:zzz");
    }

    #[test]
    fn lex_within_same_prefix_subnet() {
        let [a, b] = order_signals("subnet:192.168.0.0/16", "subnet:10.0.0.0/8");
        assert_eq!(a, "subnet:10.0.0.0/8");
        assert_eq!(b, "subnet:192.168.0.0/16");
    }

    #[test]
    fn already_sorted_duet_first() {
        let [a, b] = order_signals("duet:abc", "subnet:10.0.0.0/24");
        assert_eq!(a, "duet:abc");
        assert_eq!(b, "subnet:10.0.0.0/24");
    }
}
