/// The FTP password is typed on the camera's on-screen keyboard (owner 2026-11-16, replacing the
/// 16-character P68 password): four digits. The receiver only listens on the local network.
pub const FTP_PASSWORD_LEN: usize = 4;
/// The symbols a generated FTP password is made of.
pub const FTP_PASSWORD_ALPHABET: &[u8] = b"0123456789";

/// Rejection sampling avoids the modulo bias of mapping all 256 byte values to 10 symbols
/// (250 = 25 × 10).
pub fn ftp_password(mut rng: impl FnMut() -> u8) -> String {
    let mut password = String::with_capacity(FTP_PASSWORD_LEN);
    while password.len() < FTP_PASSWORD_LEN {
        let byte = rng();
        if byte < 250 {
            password.push(FTP_PASSWORD_ALPHABET[(byte % 10) as usize] as char);
        }
    }
    password
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c6a_password_shape_and_rejection() {
        let mut bytes = [250u8, 0, 9, 249, 255, 13].into_iter().cycle();
        let password = ftp_password(|| bytes.next().unwrap());
        assert_eq!(password, "0993");
        for seed in 0..1_000u16 {
            let mut n = seed as u32;
            let p = ftp_password(|| {
                n = n.wrapping_mul(1664525).wrapping_add(1013904223);
                (n >> 16) as u8
            });
            assert_eq!(p.len(), 4);
            assert!(p.bytes().all(|b| b.is_ascii_digit()));
        }
    }
}
