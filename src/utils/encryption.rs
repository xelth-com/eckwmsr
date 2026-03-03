use aes_gcm::{
    aead::{generic_array::typenum::{U12, U16}, Aead, KeyInit},
    aes::Aes192,
    AesGcm,
};
use crc32fast::Hasher;
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use super::smart_code::SmartTag;

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

/// AES-192-GCM with standard 12-byte nonce (for SmartTag binary encryption)
type Aes192Gcm12 = AesGcm<Aes192, U12>;

/// Encrypt a SmartTag into a QR-ready string.
///
/// Layout: `{prefix}{56 base32 chars}{iv_string}{suffix}`
/// - 19-byte payload → AES-192-GCM → 35 bytes (19 + 16 tag) → 56 Base32 chars
/// - iv_string: random Base32 of `iv_len` chars, SHA-256'd to derive 12-byte nonce
pub fn eck_binary_encrypt(
    tag: &SmartTag,
    prefix: &str,
    suffix: &str,
    iv_len: usize,
    key_hex: &str,
) -> Result<String, anyhow::Error> {
    let key = hex::decode(key_hex).map_err(|_| anyhow::anyhow!("invalid key hex"))?;
    if key.len() != 24 {
        return Err(anyhow::anyhow!("key must be 24 bytes (48 hex chars) for AES-192"));
    }

    // Generate random IV string from BASE32_CHARS alphabet
    let iv_string = random_base32_string(iv_len);

    // Derive 12-byte nonce: SHA-256(iv_string)[..12]
    let nonce_bytes = derive_nonce_from_iv(&iv_string);
    let nonce = aes_gcm::Nonce::<U12>::from_slice(&nonce_bytes);

    let cipher = Aes192Gcm12::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("cipher init: {}", e))?;

    let plaintext = tag.to_bytes();
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| anyhow::anyhow!("encrypt failed: {}", e))?;

    // 19 bytes plaintext + 16 bytes GCM tag = 35 bytes
    debug_assert_eq!(ciphertext.len(), 35);

    // 35 bytes = 7 groups of 5 bytes → 56 Base32 chars
    let encoded = to_base32(&ciphertext);
    debug_assert_eq!(encoded.len(), 56);

    let data_str = String::from_utf8(encoded)?;
    Ok(format!("{}{}{}{}", prefix, data_str, iv_string, suffix))
}

/// Decrypt a QR string back into a SmartTag.
///
/// Strips prefix/suffix, takes first 56 chars as data, remainder as iv_string.
/// This makes decryption tolerant of any IV length.
pub fn eck_binary_decrypt(
    barcode: &str,
    prefixes: &[String],
    expected_suffix: &str,
    key_hex: &str,
) -> Result<SmartTag, anyhow::Error> {
    let key = hex::decode(key_hex).map_err(|_| anyhow::anyhow!("invalid key hex"))?;
    if key.len() != 24 {
        return Err(anyhow::anyhow!("key must be 24 bytes (48 hex chars) for AES-192"));
    }

    // Match and strip prefix
    let body = prefixes
        .iter()
        .find_map(|p| barcode.strip_prefix(p.as_str()))
        .ok_or_else(|| anyhow::anyhow!("no matching prefix"))?;

    // Strip suffix
    let body = body
        .strip_suffix(expected_suffix)
        .ok_or_else(|| anyhow::anyhow!("suffix mismatch"))?;

    if body.len() < 56 {
        return Err(anyhow::anyhow!(
            "body too short: {} chars (need >= 56)",
            body.len()
        ));
    }

    let data_str = &body[..56];
    let iv_string = &body[56..];

    // Derive nonce from iv_string
    let nonce_bytes = derive_nonce_from_iv(iv_string);
    let nonce = aes_gcm::Nonce::<U12>::from_slice(&nonce_bytes);

    // Decode Base32 → 35 bytes
    let ciphertext = from_base32(data_str.as_bytes());
    if ciphertext.len() != 35 {
        return Err(anyhow::anyhow!(
            "decoded data is {} bytes, expected 35",
            ciphertext.len()
        ));
    }

    let cipher = Aes192Gcm12::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("cipher init: {}", e))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| anyhow::anyhow!("decryption failed: bad key, IV, or corrupted data"))?;

    if plaintext.len() != 19 {
        return Err(anyhow::anyhow!(
            "decrypted payload is {} bytes, expected 19",
            plaintext.len()
        ));
    }

    let mut bytes = [0u8; 19];
    bytes.copy_from_slice(&plaintext);
    Ok(SmartTag::from_bytes(&bytes))
}

/// Generate a random string of `len` characters from the BASE32_CHARS alphabet.
fn random_base32_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut buf = vec![0u8; len];
    rng.fill_bytes(&mut buf);
    buf.iter()
        .map(|&b| BASE32_CHARS[(b & 31) as usize] as char)
        .collect()
}

/// Derive a 12-byte AES-GCM nonce from an arbitrary-length IV string via SHA-256.
fn derive_nonce_from_iv(iv_string: &str) -> [u8; 12] {
    let hash = Sha256::digest(iv_string.as_bytes());
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&hash[..12]);
    nonce
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::smart_code::{ENTITY_WMS_ITEM, ENTITY_TWENTY_COMPANY};

    // Test key: 24 bytes = 48 hex chars
    const TEST_KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn test_binary_encrypt_decrypt_roundtrip() {
        let uuid = uuid::Uuid::new_v4();
        let tag = SmartTag::new(uuid, ENTITY_WMS_ITEM, 0x0042);

        let prefix = "ECK1.COM/";
        let suffix = "IB";
        let iv_len = 9;

        let encrypted = eck_binary_encrypt(&tag, prefix, suffix, iv_len, TEST_KEY).unwrap();

        assert!(encrypted.starts_with(prefix));
        assert!(encrypted.ends_with(suffix));
        assert_eq!(encrypted.len(), prefix.len() + 56 + iv_len + suffix.len());

        let prefixes = vec![prefix.to_string()];
        let decrypted = eck_binary_decrypt(&encrypted, &prefixes, suffix, TEST_KEY).unwrap();

        assert_eq!(decrypted, tag);
        assert_eq!(decrypted.uuid(), uuid);
        assert_eq!(decrypted.entity_type, ENTITY_WMS_ITEM);
        assert_eq!(decrypted.flags, 0x0042);
    }

    #[test]
    fn test_binary_different_iv_lengths() {
        let uuid = uuid::Uuid::new_v4();
        let tag = SmartTag::new(uuid, ENTITY_TWENTY_COMPANY, 0xFFFF);
        let prefix = "ECK1.COM/";
        let suffix = "IB";
        let prefixes = vec![prefix.to_string()];

        let enc5 = eck_binary_encrypt(&tag, prefix, suffix, 5, TEST_KEY).unwrap();
        assert_eq!(enc5.len(), 9 + 56 + 5 + 2);

        let enc12 = eck_binary_encrypt(&tag, prefix, suffix, 12, TEST_KEY).unwrap();
        assert_eq!(enc12.len(), 9 + 56 + 12 + 2);

        // Both decrypt correctly — decryptor auto-detects IV length
        let dec5 = eck_binary_decrypt(&enc5, &prefixes, suffix, TEST_KEY).unwrap();
        let dec12 = eck_binary_decrypt(&enc12, &prefixes, suffix, TEST_KEY).unwrap();
        assert_eq!(dec5, tag);
        assert_eq!(dec12, tag);
    }

    #[test]
    fn test_binary_multiple_prefixes() {
        let tag = SmartTag::new(uuid::Uuid::new_v4(), ENTITY_WMS_ITEM, 0);
        let prefixes = vec![
            "ECK1.COM/".to_string(),
            "ECK2.COM/".to_string(),
            "ECK3.COM/".to_string(),
        ];

        let enc = eck_binary_encrypt(&tag, "ECK2.COM/", "IB", 9, TEST_KEY).unwrap();
        let dec = eck_binary_decrypt(&enc, &prefixes, "IB", TEST_KEY).unwrap();
        assert_eq!(dec, tag);
    }

    #[test]
    fn test_binary_wrong_key_fails() {
        let tag = SmartTag::new(uuid::Uuid::new_v4(), ENTITY_WMS_ITEM, 0);
        let enc = eck_binary_encrypt(&tag, "ECK1.COM/", "IB", 9, TEST_KEY).unwrap();

        let wrong_key = "abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef";
        let prefixes = vec!["ECK1.COM/".to_string()];
        let result = eck_binary_decrypt(&enc, &prefixes, "IB", wrong_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_wrong_suffix_fails() {
        let tag = SmartTag::new(uuid::Uuid::new_v4(), ENTITY_WMS_ITEM, 0);
        let enc = eck_binary_encrypt(&tag, "ECK1.COM/", "IB", 9, TEST_KEY).unwrap();
        let prefixes = vec!["ECK1.COM/".to_string()];
        let result = eck_binary_decrypt(&enc, &prefixes, "XX", TEST_KEY);
        assert!(result.is_err());
    }

    #[test]
    fn test_base32_roundtrip_35_bytes() {
        let mut data = [0u8; 35];
        rand::thread_rng().fill_bytes(&mut data);
        let encoded = to_base32(&data);
        assert_eq!(encoded.len(), 56);
        let decoded = from_base32(&encoded);
        assert_eq!(decoded.len(), 35);
        assert_eq!(&decoded[..], &data[..]);
    }

    #[test]
    fn test_qr_version_3_fit() {
        // QR Version 3 Alphanumeric = 77 chars max
        // prefix(9) + data(56) + iv(9) + suffix(2) = 76 ≤ 77
        let tag = SmartTag::new(uuid::Uuid::new_v4(), ENTITY_WMS_ITEM, 0);
        let enc = eck_binary_encrypt(&tag, "ECK1.COM/", "IB", 9, TEST_KEY).unwrap();
        assert!(enc.len() <= 77, "QR string {} chars > 77", enc.len());
    }
}
