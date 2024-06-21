#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn log2_if_power_of_2(x: u32) -> Option<u32> {
    if x != 0 && (x & (x - 1)) == 0 {
        Some(f64::from(x).log2() as u32)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_of_2() {
        assert_eq!(log2_if_power_of_2(1), Some(0));
        assert_eq!(log2_if_power_of_2(2), Some(1));
        assert_eq!(log2_if_power_of_2(4), Some(2));
        assert_eq!(log2_if_power_of_2(8), Some(3));
        assert_eq!(log2_if_power_of_2(16), Some(4));
        assert_eq!(log2_if_power_of_2(16384), Some(14));
        assert_eq!(log2_if_power_of_2(16384 * 16384), Some(28));
    }

    #[test]
    fn test_not_power_of_2() {
        assert_eq!(log2_if_power_of_2(0), None);
        assert_eq!(log2_if_power_of_2(3), None);
        assert_eq!(log2_if_power_of_2(5), None);
        assert_eq!(log2_if_power_of_2(6), None);
        assert_eq!(log2_if_power_of_2(9), None);
        assert_eq!(log2_if_power_of_2(16383), None);
        assert_eq!(log2_if_power_of_2(16385), None);
    }

    #[test]
    fn test_large_power_of_2() {
        assert_eq!(log2_if_power_of_2(1024), Some(10));
        assert_eq!(log2_if_power_of_2(1 << 15), Some(15));
    }
}
