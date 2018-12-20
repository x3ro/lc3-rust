pub fn sign_extend(x: u16, msb: u16) -> u16 {
    // Left-pads `x` with the bit value at the bit-position indicated by `msb`. 
    if (x >> (msb - 1)) == 0 {
        return x;
    }
    return !((2 as u16).pow(msb as u32)-1) | x;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_extend_negative() {
        assert_eq!(sign_extend(0b0000_0001_0000_0000, 9), 0b1111_1111_0000_0000);
        assert_eq!(sign_extend(0b0000_0010_1010_1010, 10), 0b1111_1110_1010_1010);
        assert_eq!(sign_extend(0b0000_1000_0000_0001, 12), 0b1111_1000_0000_0001);
    }

    #[test]
    fn test_sign_extend_positive() {
        assert_eq!(sign_extend(0b0000_0000_0101_0101, 9), 0b0000_0000_0101_0101);
        assert_eq!(sign_extend(0b0000_1100_1100_1100, 13), 0b0000_1100_1100_1100);
    }
}