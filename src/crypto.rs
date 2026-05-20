use chacha20poly1305::{XChaCha20Poly1305, KeyInit};
use chacha20poly1305::aead::AeadInPlace;

pub fn encrypt_for_ntp(data: &[u8], key: &[u8; 32]) -> [u8; 8] {
    let mut output = [0u8; 8];
    let len = data.len().min(8);
    output[..len].copy_from_slice(&data[..len]);

    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = chacha20poly1305::XNonce::from_slice(&[0u8; 24]);
    let mut temp = [0u8; 8];

    let _ = cipher.encrypt_in_place_detached(nonce, b"chronos", &mut temp);

    for (i, b) in output.iter_mut().enumerate() {
        *b ^= temp[i];
    }
    output
}

pub fn decrypt_from_ntp(hidden: &[u8; 8], key: &[u8; 32]) -> Option<Vec<u8>> {
    let mut plaintext = *hidden;
    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = chacha20poly1305::XNonce::from_slice(&[0u8; 24]);
    let mut temp = [0u8; 8];

    let _ = cipher.encrypt_in_place_detached(nonce, b"chronos", &mut temp);

    for (i, b) in plaintext.iter_mut().enumerate() {
        *b ^= temp[i];
    }

    if plaintext[0] == 0 {
        None
    } else {
        Some(plaintext.to_vec())
    }
}
