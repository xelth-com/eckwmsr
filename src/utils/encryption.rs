use aes_gcm::{
    aead::{generic_array::typenum::U16, Aead, KeyInit},
    aes::Aes192,
    AesGcm,
};
use crc32fast::Hasher;
use rand::RngCore;
use std::collections::HashMap;
use std::env;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Custom Base32 alphabet (32 chars, excludes I, O, S, Z for readability)
pub const BASE32_CHARS: &[u8] = b"0123456789ABCDEFGHJKLMNPQRTUVWXY";

/// AES-192-GCM with 16-byte nonce (matching Go's cipher.NewGCMWithNonceSize(block, 16))
type Aes192Gcm16 = AesGcm<Aes192, U16>;

/// Reverse lookup table for Base32 decoding
static BASE32_BACK_HASH: LazyLock<HashMap<u8, usize>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for (i, &c) in BASE32_CHARS.iter().enumerate() {
        m.insert(c, i);
    }
    m
});

/// Convert a number (0-31) to a Base32 character
pub fn to_base32_char(num: usize) -> String {
    if num >= BASE32_CHARS.len() {
        return "?".to_string();
    }
    (BASE32_CHARS[num] as char).to_string()
}

/// Generate a time-based IV using Base32 encoding (matches Go's timeIvBase32)
fn time_iv_base32(iv_length: usize) -> Vec<u8> {
    let iv_length = if iv_length < 3 { 3 } else { iv_length };

    let temp_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
        / 7777777;

    let mut buf = vec![0u8; iv_length];

    // Fill with random bytes first (all except last 3)
    if iv_length > 3 {
        rand::thread_rng().fill_bytes(&mut buf[..iv_length - 3]);
    }

    // Convert all bytes to Base32
    for i in 0..iv_length {
        if i >= iv_length - 3 {
            let shift = 5 * (iv_length - 1 - i);
            buf[i] = BASE32_CHARS[((temp_time >> shift) & 31) as usize];
        } else {
            buf[i] = BASE32_CHARS[(buf[i] & 31) as usize];
        }
    }

    buf
}

/// Convert buffer to Base32 encoded buffer (5 bytes -> 8 chars)
fn to_base32(buf_in: &[u8]) -> Vec<u8> {
    let iterat = buf_in.len() / 5;
    let mut buf_out = vec![0u8; iterat * 8];

    for i in 0..iterat {
        buf_out[i * 8] = BASE32_CHARS[(buf_in[i * 5] >> 3) as usize];
        buf_out[i * 8 + 1] =
            BASE32_CHARS[(((buf_in[i * 5] << 2) + (buf_in[i * 5 + 1] >> 6)) & 31) as usize];
        buf_out[i * 8 + 2] = BASE32_CHARS[((buf_in[i * 5 + 1] >> 1) & 31) as usize];
        buf_out[i * 8 + 3] =
            BASE32_CHARS[(((buf_in[i * 5 + 1] << 4) + (buf_in[i * 5 + 2] >> 4)) & 31) as usize];
        buf_out[i * 8 + 4] =
            BASE32_CHARS[(((buf_in[i * 5 + 2] << 1) + (buf_in[i * 5 + 3] >> 7)) & 31) as usize];
        buf_out[i * 8 + 5] = BASE32_CHARS[((buf_in[i * 5 + 3] >> 2) & 31) as usize];
        buf_out[i * 8 + 6] =
            BASE32_CHARS[(((buf_in[i * 5 + 3] << 3) + (buf_in[i * 5 + 4] >> 5)) & 31) as usize];
        buf_out[i * 8 + 7] = BASE32_CHARS[(buf_in[i * 5 + 4] & 31) as usize];
    }

    buf_out
}

/// Decode Base32 encoded buffer back to original format (8 chars -> 5 bytes)
fn from_base32(buf_in: &[u8]) -> Vec<u8> {
    let bh = &*BASE32_BACK_HASH;
    let iterat = buf_in.len() / 8;
    let mut buf_out = vec![0u8; iterat * 5];

    for i in 0..iterat {
        let b = |idx: usize| -> u8 { *bh.get(&buf_in[i * 8 + idx]).unwrap_or(&0) as u8 };

        buf_out[i * 5] = (b(0) << 3) + (b(1) >> 2);
        buf_out[i * 5 + 1] = (b(1) << 6) + (b(2) << 1) + (b(3) >> 4);
        buf_out[i * 5 + 2] = (b(3) << 4) + (b(4) >> 1);
        buf_out[i * 5 + 3] = (b(4) << 7) + (b(5) << 2) + (b(6) >> 3);
        buf_out[i * 5 + 4] = (b(6) << 5) + b(7);
    }

    buf_out
}

/// Encrypt a string and format it for use in a URL (AES-192-GCM with custom Base32)
/// Returns the encoded ciphertext + IV string (65 chars for ECK URL usage)
pub fn eck_url_encrypt(plaintext: &str) -> Result<String, anyhow::Error> {
    let enc_key_hex = env::var("ENC_KEY").unwrap_or_default();
    if enc_key_hex.is_empty() {
        return Err(anyhow::anyhow!("ENC_KEY environment variable not set"));
    }

    let key = hex::decode(&enc_key_hex).map_err(|_| anyhow::anyhow!("invalid ENC_KEY format"))?;

    if key.len() != 24 {
        return Err(anyhow::anyhow!(
            "ENC_KEY must be 24 bytes (48 hex chars) for AES-192"
        ));
    }

    let cipher = Aes192Gcm16::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("cipher init error: {}", e))?;

    // Generate time-based IV (9 bytes Base32)
    let bet_iv = time_iv_base32(9);

    // Create full 16-byte IV by concatenating (matching Go/Node.js behavior)
    let mut iv = vec![0u8; 16];
    iv[..9].copy_from_slice(&bet_iv);
    iv[9..].copy_from_slice(&bet_iv[..7]);

    let nonce = aes_gcm::Nonce::<U16>::from_slice(&iv);

    // Encrypt (ciphertext includes auth tag at the end, 16 bytes for GCM)
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("encryption failed: {}", e))?;

    // Encode to Base32
    let base32_encoded = to_base32(&ciphertext);

    // Combine Base32 data + Base32 IV
    let mut result = base32_encoded;
    result.extend_from_slice(&bet_iv);

    Ok(String::from_utf8(result)?)
}

/// Decrypt a URL-formatted encrypted string
pub fn eck_url_decrypt(encrypted_url: &str) -> Result<String, anyhow::Error> {
    let instance_suffix = env::var("INSTANCE_SUFFIX").unwrap_or_default();
    if instance_suffix.is_empty() {
        return Err(anyhow::anyhow!(
            "INSTANCE_SUFFIX environment variable not set"
        ));
    }

    if encrypted_url.len() != 76 {
        return Err(anyhow::anyhow!("invalid encrypted URL length"));
    }

    let prefix = &encrypted_url[0..9];
    if prefix != "ECK1.COM/" && prefix != "ECK2.COM/" && prefix != "ECK3.COM/" {
        return Err(anyhow::anyhow!("invalid URL prefix"));
    }

    let suffix = &encrypted_url[74..76];
    if suffix != instance_suffix {
        return Err(anyhow::anyhow!("invalid instance suffix"));
    }

    let enc_key_hex = env::var("ENC_KEY").unwrap_or_default();
    if enc_key_hex.is_empty() {
        return Err(anyhow::anyhow!("ENC_KEY environment variable not set"));
    }

    let key = hex::decode(&enc_key_hex).map_err(|_| anyhow::anyhow!("invalid ENC_KEY format"))?;
    if key.len() != 24 {
        return Err(anyhow::anyhow!("ENC_KEY must be 24 bytes for AES-192"));
    }

    // Extract Base32 IV (chars 65-74) and Base32 data (chars 9-65)
    let bet_iv = encrypted_url[65..74].as_bytes();
    let base32_data = encrypted_url[9..65].as_bytes();

    // Decode from Base32
    let decoded_data = from_base32(base32_data);

    // Reconstruct IV
    let mut iv = vec![0u8; 16];
    iv[..9].copy_from_slice(bet_iv);
    iv[9..].copy_from_slice(&bet_iv[..7]);

    let cipher = Aes192Gcm16::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("cipher init error: {}", e))?;

    let nonce = aes_gcm::Nonce::<U16>::from_slice(&iv);

    let plaintext = cipher
        .decrypt(nonce, decoded_data.as_ref())
        .map_err(|_| anyhow::anyhow!("decryption failed: invalid auth tag or corrupted data"))?;

    Ok(String::from_utf8(plaintext)?)
}

/// Generate a 2-char CRC check value from an integer (using its decimal string representation)
/// This matches the Go generator's inline CRC: crc32.ChecksumIEEE([]byte(fmt.Sprintf("%d", id))) & 1023
pub fn eck_crc(value: i64) -> String {
    let mut hasher = Hasher::new();
    hasher.update(value.to_string().as_bytes());
    let temp = hasher.finalize() & 1023;
    let char1 = BASE32_CHARS[(temp >> 5) as usize] as char;
    let char2 = BASE32_CHARS[(temp & 31) as usize] as char;
    format!("{}{}", char1, char2)
}

/// Encrypt a string using AES-192-GCM and return hex-encoded result (standard nonce)
pub fn encrypt_string(plaintext: &str) -> Result<String, anyhow::Error> {
    let enc_key_hex = env::var("ENC_KEY").unwrap_or_default();
    if enc_key_hex.is_empty() {
        return Err(anyhow::anyhow!("ENC_KEY environment variable not set"));
    }

    let key = hex::decode(&enc_key_hex).map_err(|_| anyhow::anyhow!("invalid ENC_KEY format"))?;
    if key.len() != 24 {
        return Err(anyhow::anyhow!(
            "ENC_KEY must be 24 bytes (48 hex chars) for AES-192"
        ));
    }

    // Standard AES-192-GCM with 12-byte nonce
    use aes_gcm::aead::generic_array::typenum::U12;
    type Aes192Gcm12 = AesGcm<Aes192, U12>;

    let cipher = Aes192Gcm12::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("cipher init error: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = aes_gcm::Nonce::<U12>::from_slice(&nonce_bytes);

    // Encrypt and prepend nonce
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("encryption failed: {}", e))?;

    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);

    Ok(hex::encode(result))
}

/// Decrypt a hex-encoded encrypted string
pub fn decrypt_string(encrypted_hex: &str) -> Result<String, anyhow::Error> {
    let enc_key_hex = env::var("ENC_KEY").unwrap_or_default();
    if enc_key_hex.is_empty() {
        return Err(anyhow::anyhow!("ENC_KEY environment variable not set"));
    }

    let key = hex::decode(&enc_key_hex).map_err(|_| anyhow::anyhow!("invalid ENC_KEY format"))?;
    if key.len() != 24 {
        return Err(anyhow::anyhow!("ENC_KEY must be 24 bytes for AES-192"));
    }

    let data =
        hex::decode(encrypted_hex).map_err(|_| anyhow::anyhow!("invalid encrypted data format"))?;

    use aes_gcm::aead::generic_array::typenum::U12;
    type Aes192Gcm12 = AesGcm<Aes192, U12>;

    let cipher = Aes192Gcm12::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("cipher init error: {}", e))?;

    if data.len() < 12 {
        return Err(anyhow::anyhow!("ciphertext too short"));
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = aes_gcm::Nonce::<U12>::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("decryption failed"))?;

    Ok(String::from_utf8(plaintext)?)
}
