/// The FTP password is typed on the camera's on-screen keyboard (P68, owner 2026-11-11):
/// lowercase letters and digits without the look-alikes `0 o 1 l i`, 16 characters (~79 bits).
pub const FTP_PASSWORD_LEN: usize = 16;
/// The 31 symbols a generated FTP password is made of.
pub const FTP_PASSWORD_ALPHABET: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";

/// Rejection sampling avoids the modulo bias of mapping all 256 byte values to 31 symbols
/// (248 = 8 × 31).
pub fn ftp_password(mut rng: impl FnMut() -> u8) -> String {
    let mut password = String::with_capacity(FTP_PASSWORD_LEN);
    while password.len() < FTP_PASSWORD_LEN {
        let byte = rng();
        if byte < 248 {
            password.push(FTP_PASSWORD_ALPHABET[(byte % 31) as usize] as char);
        }
    }
    password
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c6a_password_shape_and_rejection() {
        let mut bytes = [248u8, 0, 30, 247].into_iter().cycle();
        let password = ftp_password(|| bytes.next().unwrap());
        assert_eq!(password, "a99a99a99a99a99a");
        for seed in 0..1_000u16 {
            let mut n = seed as u32;
            let p = ftp_password(|| {
                n = n.wrapping_mul(1664525).wrapping_add(1013904223);
                (n >> 16) as u8
            });
            assert_eq!(p.len(), 16);
            assert!(p.bytes().all(|b| FTP_PASSWORD_ALPHABET.contains(&b)));
            assert!(!p.contains(['0', 'o', '1', 'l', 'i']));
        }
    }
}
