//! Thai enterprise utilities: BahtText currency converter and National ID validation.

/// Converts a numeric amount to Thai text format (similar to Microsoft Excel BAHTTEXT).
///
/// # Examples
/// ```
/// use thai_break_service::utils::bahttext;
/// assert_eq!(bahttext(1.0), "หนึ่งบาทถ้วน");
/// assert_eq!(bahttext(21.50), "ยี่สิบเอ็ดบาทห้าสิบสตางค์");
/// assert_eq!(bahttext(0.25), "ยี่สิบห้าสตางค์");
/// assert_eq!(bahttext(0.0), "ศูนย์บาทถ้วน");
/// assert_eq!(bahttext(-50.0), "ลบห้าสิบบาทถ้วน");
/// ```
pub fn bahttext(amount: f64) -> String {
    if amount.is_nan() || amount.is_infinite() {
        return String::new();
    }

    let is_negative = amount < 0.0;
    let abs_amount = amount.abs();

    // Round to 2 decimal places
    let total_satang = (abs_amount * 100.0).round() as i64;
    let baht = total_satang / 100;
    let satang = total_satang % 100;

    if baht == 0 && satang == 0 {
        return "ศูนย์บาทถ้วน".to_string();
    }

    let mut result = String::new();
    if is_negative {
        result.push_str("ลบ");
    }

    if baht > 0 {
        result.push_str(&num_to_thai(baht));
        result.push_str("บาท");
    }

    if satang > 0 {
        result.push_str(&num_to_thai(satang));
        result.push_str("สตางค์");
    } else {
        result.push_str("ถ้วน");
    }

    result
}

/// Parse and convert arbitrary numeric string (e.g. "1,234,567.89") to BahtText.
pub fn bahttext_str(input: &str) -> Result<String, &'static str> {
    let clean: String = input.chars().filter(|&c| c != ',' && c != ' ').collect();
    if clean.is_empty() {
        return Err("Input string is empty");
    }

    let is_negative = clean.starts_with('-');
    let raw = clean.trim_start_matches('-').trim_start_matches('+');

    let parts: Vec<&str> = raw.split('.').collect();
    if parts.len() > 2 {
        return Err("Invalid decimal format: multiple decimal points");
    }

    let baht_str = parts[0];
    if !baht_str.chars().all(|c| c.is_ascii_digit()) {
        return Err("Invalid characters in baht portion");
    }

    let baht: i64 = if baht_str.is_empty() {
        0
    } else {
        baht_str.parse().map_err(|_| "Baht amount out of range")?
    };

    let satang: i64 = if parts.len() == 2 {
        let dec_str = parts[1];
        if !dec_str.chars().all(|c| c.is_ascii_digit()) {
            return Err("Invalid characters in satang portion");
        }
        if dec_str.is_empty() {
            0
        } else if dec_str.len() == 1 {
            let d: i64 = dec_str.parse().unwrap();
            d * 10
        } else {
            let two_digits = &dec_str[..2];
            let mut s: i64 = two_digits.parse().unwrap();
            // Round up if 3rd digit >= 5
            if dec_str.len() > 2 {
                let third_digit = dec_str.as_bytes()[2] - b'0';
                if third_digit >= 5 {
                    s += 1;
                }
            }
            s
        }
    } else {
        0
    };

    let mut total_baht = baht;
    let mut final_satang = satang;
    if final_satang >= 100 {
        total_baht += final_satang / 100;
        final_satang %= 100;
    }

    if total_baht == 0 && final_satang == 0 {
        return Ok("ศูนย์บาทถ้วน".to_string());
    }

    let mut result = String::new();
    if is_negative {
        result.push_str("ลบ");
    }

    if total_baht > 0 {
        result.push_str(&num_to_thai(total_baht));
        result.push_str("บาท");
    }

    if final_satang > 0 {
        result.push_str(&num_to_thai(final_satang));
        result.push_str("สตางค์");
    } else {
        result.push_str("ถ้วน");
    }

    Ok(result)
}

/// Convert integer up to millions/billions into Thai text.
fn num_to_thai(num: i64) -> String {
    if num == 0 {
        return "ศูนย์".to_string();
    }

    let mut n = num.abs();
    let mut groups = Vec::new();

    while n > 0 {
        groups.push((n % 1_000_000) as usize);
        n /= 1_000_000;
    }

    let mut out = String::new();
    for (idx, &group_val) in groups.iter().enumerate().rev() {
        if group_val > 0 {
            let is_not_first = idx < groups.len() - 1;
            out.push_str(&block_to_thai(group_val, is_not_first));
            if idx > 0 {
                out.push_str("ล้าน");
            }
        }
    }

    out
}

/// Convert a number < 1,000,000 into Thai text.
fn block_to_thai(val: usize, has_higher_block: bool) -> String {
    const DIGITS: [&str; 10] = ["", "หนึ่ง", "สอง", "สาม", "สี่", "ห้า", "หก", "เจ็ด", "แปด", "เก้า"];
    const PLACES: [&str; 6] = ["", "สิบ", "ร้อย", "พัน", "หมื่น", "แสน"];

    let mut digits = [0usize; 6];
    let mut tmp = val;
    for d in digits.iter_mut() {
        *d = tmp % 10;
        tmp /= 10;
    }

    let mut res = String::new();
    let has_tens_or_higher = digits[1..].iter().any(|&d| d > 0);

    for place in (0..6).rev() {
        let d = digits[place];
        if d == 0 {
            continue;
        }

        if place == 1 {
            // Tens place
            if d == 1 {
                res.push_str("สิบ");
            } else if d == 2 {
                res.push_str("ยี่สิบ");
            } else {
                res.push_str(DIGITS[d]);
                res.push_str("สิบ");
            }
        } else if place == 0 {
            // Units place
            if d == 1 && (has_tens_or_higher || has_higher_block) {
                res.push_str("เอ็ด");
            } else {
                res.push_str(DIGITS[d]);
            }
        } else {
            res.push_str(DIGITS[d]);
            res.push_str(PLACES[place]);
        }
    }

    res
}

/// Result of Thai National ID validation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ThaiIdResult {
    pub is_valid: bool,
    pub id: String,
    pub formatted: Option<String>,
    pub error: Option<String>,
}

/// Validate a 13-digit Thai Citizen ID using the official DOPA checksum algorithm.
pub fn validate_thai_id(input: &str) -> ThaiIdResult {
    let clean: String = input.chars().filter(|c| c.is_ascii_digit()).collect();

    if clean.len() != 13 {
        return ThaiIdResult {
            is_valid: false,
            id: input.to_string(),
            formatted: None,
            error: Some(format!("Invalid length: expected 13 digits, found {}", clean.len())),
        };
    }

    let digits: Vec<u32> = clean.chars().map(|c| c.to_digit(10).unwrap()).collect();

    // Checksum: sum(d_i * (13 - i)) for i in 0..12
    let sum: u32 = digits[..12]
        .iter()
        .enumerate()
        .map(|(i, &d)| d * (13 - i as u32))
        .sum();

    let check_digit = (11 - (sum % 11)) % 10;
    let is_valid = check_digit == digits[12];

    let formatted = format!(
        "{}-{}{}{}{}-{}{}{}{}{}-{}{}-{}",
        digits[0],
        digits[1], digits[2], digits[3], digits[4],
        digits[5], digits[6], digits[7], digits[8], digits[9],
        digits[10], digits[11],
        digits[12]
    );

    ThaiIdResult {
        is_valid,
        id: clean,
        formatted: Some(formatted),
        error: if is_valid {
            None
        } else {
            Some(format!(
                "Invalid checksum: expected check digit {}, got {}",
                check_digit, digits[12]
            ))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bahttext_basic() {
        assert_eq!(bahttext(0.0), "ศูนย์บาทถ้วน");
        assert_eq!(bahttext(1.0), "หนึ่งบาทถ้วน");
        assert_eq!(bahttext(11.0), "สิบเอ็ดบาทถ้วน");
        assert_eq!(bahttext(21.0), "ยี่สิบเอ็ดบาทถ้วน");
        assert_eq!(bahttext(25.50), "ยี่สิบห้าบาทห้าสิบสตางค์");
        assert_eq!(bahttext(100.0), "หนึ่งร้อยบาทถ้วน");
        assert_eq!(bahttext(101.0), "หนึ่งร้อยเอ็ดบาทถ้วน");
        assert_eq!(bahttext(1001.0), "หนึ่งพันเอ็ดบาทถ้วน");
        assert_eq!(bahttext(1_000_000.0), "หนึ่งล้านบาทถ้วน");
        assert_eq!(bahttext(1_000_001.0), "หนึ่งล้านเอ็ดบาทถ้วน");
        assert_eq!(bahttext(10_000_000.0), "สิบล้านบาทถ้วน");
        assert_eq!(
            bahttext(1_234_567.89),
            "หนึ่งล้านสองแสนสามหมื่นสี่พันห้าร้อยหกสิบเจ็ดบาทแปดสิบเก้าสตางค์"
        );
        assert_eq!(bahttext(0.25), "ยี่สิบห้าสตางค์");
        assert_eq!(bahttext(0.75), "เจ็ดสิบห้าสตางค์");
        assert_eq!(bahttext(-50.0), "ลบห้าสิบบาทถ้วน");
    }

    #[test]
    fn test_bahttext_str() {
        assert_eq!(bahttext_str("1,234.50").unwrap(), "หนึ่งพันสองร้อยสามสิบสี่บาทห้าสิบสตางค์");
        assert_eq!(bahttext_str("0.25").unwrap(), "ยี่สิบห้าสตางค์");
        assert_eq!(bahttext_str("100").unwrap(), "หนึ่งร้อยบาทถ้วน");
        assert_eq!(bahttext_str("-500.00").unwrap(), "ลบห้าร้อยบาทถ้วน");
        assert!(bahttext_str("abc").is_err());
    }

    #[test]
    fn test_validate_thai_id() {
        // Valid ID: 1234567890121
        let valid = validate_thai_id("1234567890121");
        assert!(valid.is_valid);
        assert_eq!(valid.formatted.unwrap(), "1-2345-67890-12-1");
        assert!(valid.error.is_none());

        // Valid ID with dashes
        let valid_dashes = validate_thai_id("1-2345-67890-12-1");
        assert!(valid_dashes.is_valid);

        // Invalid checksum
        let invalid_check = validate_thai_id("1234567890120");
        assert!(!invalid_check.is_valid);
        assert!(invalid_check.error.unwrap().contains("Invalid checksum"));

        // Invalid length
        let invalid_len = validate_thai_id("12345");
        assert!(!invalid_len.is_valid);
        assert!(invalid_len.error.unwrap().contains("Invalid length"));
    }
}
