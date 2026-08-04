pub fn wrapping_lt(a: u16, b: u16) -> bool {
    (a.wrapping_sub(b) as i16) < 0
}

pub fn wrapping_le(a: u16, b: u16) -> bool {
    a == b || (a.wrapping_sub(b) as i16) < 0
}

pub fn wrapping_gt(a: u16, b: u16) -> bool {
    (a.wrapping_sub(b) as i16) > 0
}

pub fn wrapping_ge(a: u16, b: u16) -> bool {
    a == b || (a.wrapping_sub(b) as i16) > 0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn wrapping_less_than() {
        assert!(wrapping_lt(0, 1));
        assert!(wrapping_lt(0, u16::MAX / 2));
        assert!(wrapping_lt(u16::MAX, 0));
        assert!(wrapping_lt(u16::MAX / 2 + 1, 0));
        assert!(wrapping_lt(u16::MAX / 2, u16::MAX / 2 + 1));
    }

    #[test]
    fn wrapping_less_than_or_equal() {
        assert!(wrapping_le(0, 1));
        assert!(wrapping_le(0, u16::MAX / 2));
        assert!(wrapping_le(u16::MAX, 0));
        assert!(wrapping_le(u16::MAX / 2 + 1, 0));
        assert!(wrapping_le(u16::MAX / 2, u16::MAX / 2 + 1));
        assert!(wrapping_le(0, 0));
        assert!(wrapping_le(u16::MAX, u16::MAX));
    }

    #[test]
    fn wrapping_greater_than() {
        assert!(wrapping_gt(2, 1));
        assert!(wrapping_gt(u16::MAX, u16::MAX / 2 + 1));
        assert!(wrapping_gt(u16::MAX / 2, 0));
        assert!(!wrapping_gt(u16::MAX / 2 + 1, 0));
        assert!(!wrapping_gt(u16::MAX / 2, u16::MAX / 2));
    }

    #[test]
    fn wrapping_greater_than_or_equal() {
        assert!(wrapping_ge(2, 1));
        assert!(wrapping_ge(u16::MAX, u16::MAX / 2 + 1));
        assert!(wrapping_ge(u16::MAX / 2, 0));
        assert!(!wrapping_ge(u16::MAX / 2 + 1, 0));
        assert!(wrapping_ge(21_000, 21_000));
        assert!(wrapping_ge(u16::MAX / 2, u16::MAX / 2));
    }
}
