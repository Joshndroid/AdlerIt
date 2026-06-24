pub fn adler32(bytes: &[u8]) -> u32 {
    adler2::adler32_slice(bytes)
}

pub fn hex(value: u32) -> String {
    format!("{value:08x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vectors() {
        assert_eq!(adler32(b""), 1);
        assert_eq!(hex(adler32(b"Wikipedia")), "11e60398");
        assert_eq!(hex(adler32(b"123456789")), "091e01de");
    }
}
