use chrono::{DateTime, TimeZone, Utc, Duration};
use thiserror::Error;

// Base36 alphabet (0-9, A-Z) — matches Go's SmartBase32Chars
const SMART_BASE32_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Error, Debug)]
pub enum SmartCodeError {
    #[error("Invalid item code")]
    InvalidItemCode,
    #[error("Code too short for specified split length")]
    CodeTooShort,
    #[error("Invalid box code")]
    InvalidBoxCode,
    #[error("Dimensions too large")]
    DimensionsTooLarge,
    #[error("Weight too heavy")]
    WeightTooHeavy,
    #[error("Invalid label code")]
    InvalidLabelCode,
    #[error("Date too old")]
    DateTooOld,
    #[error("Date too far future")]
    DateTooFarFuture,
    #[error("Invalid place code")]
    InvalidPlaceCode,
    #[error("Parse error")]
    ParseError,
}

// Weight Tiers (matches Go constants exactly)
const TIER1_LIMIT: f64 = 20.0;
const TIER1_STEP: f64 = 0.01;
const TIER1_MAX: usize = 2000;
const TIER2_LIMIT: f64 = 1000.0;
const TIER2_STEP: f64 = 0.1;
const TIER2_MAX: usize = 11800;
const TIER3_LIMIT: f64 = 30000.0;
const TIER3_STEP: f64 = 1.0;

fn smart_label_epoch() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()
}

// ==========================================
// Data Structures
// ==========================================

pub struct SmartItemData {
    pub serial: String,
    pub ref_id: String,
}

pub struct SmartBoxData {
    pub length: usize,
    pub width: usize,
    pub height: usize,
    pub weight: f64,
    pub pkg_type: String,
    pub serial: u64,
}

pub struct SmartLabelData {
    pub date: DateTime<Utc>,
    pub label_type: String,
    pub payload: String,
}

pub struct SmartPlaceData {
    pub location_id: i64,
}

// ==========================================
// Base32 Helpers
// ==========================================

fn base32_to_int(chunk: &str) -> usize {
    let mut val = 0;
    let base = SMART_BASE32_CHARS.len();
    for ch in chunk.chars() {
        let idx = SMART_BASE32_CHARS
            .iter()
            .position(|&c| c == ch as u8)
            .unwrap_or(0);
        val = val * base + idx;
    }
    val
}

fn int_to_base32(mut num: usize, width: usize) -> String {
    let base = SMART_BASE32_CHARS.len();
    let mut res = String::new();
    for _ in 0..width {
        let rem = num % base;
        res.insert(0, SMART_BASE32_CHARS[rem] as char);
        num /= base;
    }
    res
}

fn pad_right(s: &str, length: usize, pad: char) -> String {
    if s.len() >= length {
        return s[..length].to_string();
    }
    let mut result = s.to_string();
    while result.len() < length {
        result.push(pad);
    }
    result
}

// ==========================================
// Weight Encoding (Tiered Precision)
// ==========================================

fn encode_weight(kg: f64) -> Result<usize, SmartCodeError> {
    if kg <= TIER1_LIMIT {
        Ok((kg / TIER1_STEP).round() as usize)
    } else if kg <= TIER2_LIMIT {
        Ok(TIER1_MAX + ((kg - TIER1_LIMIT) / TIER2_STEP).round() as usize)
    } else if kg <= TIER3_LIMIT {
        Ok(TIER2_MAX + ((kg - TIER2_LIMIT) / TIER3_STEP).round() as usize)
    } else {
        Err(SmartCodeError::WeightTooHeavy)
    }
}

fn decode_weight(val: usize) -> f64 {
    if val <= TIER1_MAX {
        val as f64 * TIER1_STEP
    } else if val <= TIER2_MAX {
        TIER1_LIMIT + (val - TIER1_MAX) as f64 * TIER2_STEP
    } else {
        TIER2_LIMIT + (val - TIER2_MAX) as f64 * TIER3_STEP
    }
}

// ==========================================
// 1. SMART ITEM ('i')
// Format: i [SplitChar] [Serial...] [EAN...]
// ==========================================

pub fn decode_smart_item(code: &str) -> Result<SmartItemData, SmartCodeError> {
    if code.len() < 3 || !code.starts_with('i') {
        return Err(SmartCodeError::InvalidItemCode);
    }
    let code = code.to_uppercase();

    let split_char = &code[1..2];
    let suffix_len = base32_to_int(split_char);
    let data_part = &code[2..];

    if data_part.len() < suffix_len {
        return Err(SmartCodeError::CodeTooShort);
    }

    let split_idx = data_part.len() - suffix_len;
    let serial = data_part[..split_idx].to_string();
    let ref_id = data_part[split_idx..].to_string();

    Ok(SmartItemData { serial, ref_id })
}

pub fn generate_smart_item(mut serial: String, mut ref_id: String) -> String {
    let mut suffix_len = ref_id.len();
    if suffix_len > 17 {
        ref_id.truncate(17);
        suffix_len = 17;
    }
    if suffix_len > 35 {
        suffix_len = 35;
    }

    let split_char = int_to_base32(suffix_len, 1);
    let target_serial_len = 17 - suffix_len;

    if serial.len() < target_serial_len {
        serial = format!("{}{}", "0".repeat(target_serial_len - serial.len()), serial);
    } else if serial.len() > target_serial_len {
        serial.truncate(target_serial_len);
    }

    format!("i{}{}{}", split_char, serial, ref_id)
}

// ==========================================
// 2. SMART BOX ('b')
// Format: b LL WW HH MMM T SSSSSSSS
// ==========================================

pub fn decode_smart_box(code: &str) -> Result<SmartBoxData, SmartCodeError> {
    if code.len() != 19 || !code.starts_with('b') {
        return Err(SmartCodeError::InvalidBoxCode);
    }
    let code = code.to_uppercase();

    let l = base32_to_int(&code[1..3]);
    let w = base32_to_int(&code[3..5]);
    let h = base32_to_int(&code[5..7]);
    let m_val = base32_to_int(&code[7..10]);
    let t = code[10..11].to_string();
    let serial = base32_to_int(&code[11..19]);

    Ok(SmartBoxData {
        length: l,
        width: w,
        height: h,
        weight: decode_weight(m_val),
        pkg_type: t,
        serial: serial as u64,
    })
}

pub fn generate_smart_box(data: &SmartBoxData) -> Result<String, SmartCodeError> {
    if data.length > 1023 || data.width > 1023 || data.height > 1023 {
        return Err(SmartCodeError::DimensionsTooLarge);
    }
    let m_val = encode_weight(data.weight)?;
    let t_char = data
        .pkg_type
        .chars()
        .next()
        .unwrap_or('B')
        .to_uppercase()
        .to_string();

    Ok(format!(
        "b{}{}{}{}{}{}",
        int_to_base32(data.length, 2),
        int_to_base32(data.width, 2),
        int_to_base32(data.height, 2),
        int_to_base32(m_val, 3),
        t_char,
        int_to_base32(data.serial as usize, 8)
    ))
}

// ==========================================
// 3. SMART LABEL ('l')
// Format: l DDD T SSSSSSSSSSSSSS
// ==========================================

pub fn decode_smart_label(code: &str) -> Result<SmartLabelData, SmartCodeError> {
    if code.len() != 19 || !code.starts_with('l') {
        return Err(SmartCodeError::InvalidLabelCode);
    }
    let code = code.to_uppercase();

    let days = base32_to_int(&code[1..4]);
    let date = smart_label_epoch() + Duration::days(days as i64);
    let t = code[4..5].to_string();
    let payload = code[5..].to_string();

    Ok(SmartLabelData {
        date,
        label_type: t,
        payload,
    })
}

pub fn generate_smart_label(data: &SmartLabelData) -> Result<String, SmartCodeError> {
    if data.date < smart_label_epoch() {
        return Err(SmartCodeError::DateTooOld);
    }
    let days = (data.date - smart_label_epoch()).num_days();
    if days > 46655 {
        // 36^3 - 1
        return Err(SmartCodeError::DateTooFarFuture);
    }

    let t_char = data
        .label_type
        .chars()
        .next()
        .unwrap_or('A')
        .to_uppercase()
        .to_string();

    Ok(format!(
        "l{}{}{}",
        int_to_base32(days as usize, 3),
        t_char,
        pad_right(&data.payload, 14, '0')
    ))
}

// ==========================================
// 4. SMART PLACE ('p')
// Format: p + 18-digit zero-padded Odoo location ID
// ==========================================

pub fn decode_smart_place(code: &str) -> Result<SmartPlaceData, SmartCodeError> {
    if code.len() != 19 || !code.starts_with('p') {
        return Err(SmartCodeError::InvalidPlaceCode);
    }

    let id_str = code[1..].trim_start_matches('0');
    if id_str.is_empty() {
        return Err(SmartCodeError::InvalidPlaceCode);
    }

    let id = id_str
        .parse::<i64>()
        .map_err(|_| SmartCodeError::ParseError)?;
    Ok(SmartPlaceData { location_id: id })
}

pub fn generate_smart_place(location_id: i64) -> String {
    format!("p{:018}", location_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_roundtrip() {
        let code = generate_smart_item("LOT001".to_string(), "4260123456789".to_string());
        assert_eq!(code.len(), 19);
        assert!(code.starts_with('i'));

        let decoded = decode_smart_item(&code).unwrap();
        assert_eq!(decoded.ref_id, "4260123456789");
    }

    #[test]
    fn test_box_roundtrip() {
        let data = SmartBoxData {
            length: 60,
            width: 40,
            height: 30,
            weight: 5.5,
            pkg_type: "B".to_string(),
            serial: 12345,
        };
        let code = generate_smart_box(&data).unwrap();
        assert_eq!(code.len(), 19);

        let decoded = decode_smart_box(&code).unwrap();
        assert_eq!(decoded.length, 60);
        assert_eq!(decoded.width, 40);
        assert_eq!(decoded.height, 30);
        assert!((decoded.weight - 5.5).abs() < 0.02);
    }

    #[test]
    fn test_place_roundtrip() {
        let code = generate_smart_place(31);
        assert_eq!(code, "p000000000000000031");
        let decoded = decode_smart_place(&code).unwrap();
        assert_eq!(decoded.location_id, 31);
    }

    #[test]
    fn test_weight_tiers() {
        // Tier 1: fine precision
        let enc = encode_weight(10.5).unwrap();
        let dec = decode_weight(enc);
        assert!((dec - 10.5).abs() < 0.02);

        // Tier 2: medium precision
        let enc = encode_weight(500.0).unwrap();
        let dec = decode_weight(enc);
        assert!((dec - 500.0).abs() < 0.2);

        // Tier 3: coarse precision
        let enc = encode_weight(5000.0).unwrap();
        let dec = decode_weight(enc);
        assert!((dec - 5000.0).abs() < 1.5);
    }
}
