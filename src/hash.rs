pub fn adler32(bytes: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65_521;

    let mut a = 1;
    let mut b = 0;

    for byte in bytes {
        a = (a + u32::from(*byte)) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }

    (b << 16) | a
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
