pub use chacha20poly1305::{AeadInPlace, ChaCha20Poly1305, KeyInit, Nonce};
pub use sha2::Sha256;

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const PBKDF2_ROUNDS: u32 = 600_000;

use alloc::vec::Vec;

pub fn read_key_and_decrypt(payload: &[u8]) -> Result<Vec<u8>, ()> {
    let mut key = [0; 256];
    let len = read_key(&mut key);

    decrypt(payload, &key[..len])
}

fn decrypt(payload: &[u8], key: &[u8]) -> Result<Vec<u8>, ()> {
    let salt = &payload[..SALT_LEN];
    let nonce = &payload[SALT_LEN..SALT_LEN + NONCE_LEN];
    let mut result = payload[SALT_LEN + NONCE_LEN..].to_vec();

    let mut buf = [0; 32];
    pbkdf2::pbkdf2_hmac::<Sha256>(&key, salt, PBKDF2_ROUNDS, &mut buf);

    #[allow(deprecated)]
    let cipher = ChaCha20Poly1305::new(chacha20poly1305::Key::from_slice(&buf));
    #[allow(deprecated)]
    cipher
        .decrypt_in_place(Nonce::from_slice(&nonce), b"", &mut result)
        .map_err(|_| ())?;

    Ok(result)
}

fn read_key(buf: &mut [u8]) -> usize {
    unsafe {
        libc::write(1, "Decryption key: ".as_ptr() as *const _, 16);

        let mut term: libc::termios = core::mem::zeroed();
        libc::tcgetattr(0, &mut term);
        let old_term = term;
        term.c_lflag &= !libc::ECHO;
        libc::tcsetattr(0, libc::TCSANOW, &term);

        let mut total = 0;
        while total < buf.len() {
            let mut byte: u8 = 0;
            let r = libc::read(0, &mut byte as *mut u8 as *mut _, 1);
            if r <= 0 {
                break;
            }

            if byte == b'\n' || byte == b'\r' {
                break;
            }

            buf[total] = byte;
            total += 1;
        }

        libc::tcsetattr(0, libc::TCSANOW, &old_term);

        libc::write(1, "\n".as_ptr() as *const _, 1);
        total
    }
}
