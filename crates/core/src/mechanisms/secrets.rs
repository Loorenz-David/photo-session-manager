/// FTP credentials use 62 printable alphanumeric symbols (10E R-I21).
pub const FTP_PASSWORD_LEN: usize = 32;
const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/// Rejection sampling avoids the modulo bias of mapping all 256 byte values to 62 symbols.
pub fn ftp_password(mut rng: impl FnMut() -> u8) -> String {
    let mut password = String::with_capacity(FTP_PASSWORD_LEN);
    while password.len() < FTP_PASSWORD_LEN {
        let byte = rng();
        if byte < 248 {
            password.push(ALPHABET[(byte % 62) as usize] as char);
        }
    }
    password
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c6a_password_shape_and_rejection() {
        let mut bytes = [248u8, 0, 61, 247].into_iter().cycle();
        let password = ftp_password(|| bytes.next().unwrap());
        assert_eq!(password, "A99A99A99A99A99A99A99A99A99A99A9");
        for seed in 0..1_000u16 {
            let mut n = seed as u32;
            let p = ftp_password(|| {
                n = n.wrapping_mul(1664525).wrapping_add(1013904223);
                (n >> 16) as u8
            });
            assert_eq!(p.len(), 32);
            assert!(p.bytes().all(|b| b.is_ascii_alphanumeric()));
        }
    }
}
