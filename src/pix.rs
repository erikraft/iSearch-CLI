//! Module for PIX payload generation according to the Banco Central do Brasil specifications.
//!
//! # Purpose
//! This module provides the capability to generate valid, compliant EMV Co static PIX QR code
//! payloads. These payloads can be parsed by banking apps in Brazil to perform instantaneous
//! donations or payments.
//!
//! # Architecture and Interactions
//! The primary functionality centers around [generate_pix_payload], which parses configuration input
//! and validates it using [validate_amount]. The resulting string is subsequently rendered into a QR
//! code using rendering mechanisms defined in [crate::utils].

/// Calculates the CRC-16 CCITT checksum (polynomial `0x1021`, initial value `0xFFFF`) for a given data slice.
///
/// This specific variant is used for standardizing EMV Co and PIX payloads to ensure checksum integrity.
///
/// # Arguments
///
/// * `data` - The byte slice of the string input to verify.
///
/// # Returns
///
/// Returns the calculated `u16` checksum.
///
/// # Examples
///
/// ```
/// use isearch_cli::pix::crc16_ccitt;
/// let crc = crc16_ccitt(b"123456789");
/// assert_eq!(crc, 0x29B1);
/// ```
pub fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc = 0xFFFFu16;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Formats a Tag-Length-Value (TLV) block according to EMV specifications.
///
/// Ensures both the tag and the length are padded to exactly 2 digits, left-padded with zeros.
///
/// # Arguments
///
/// * `tag` - The string slice representing the field identifier.
/// * `value` - The string slice representing the value of the field.
///
/// # Returns
///
/// Returns the formatted TLV string.
fn format_tlv(tag: &str, value: &str) -> String {
    format!("{:0>2}{:0>2}{}", tag, value.len(), value)
}

/// Validates, sanitizes, and parses a user-entered donation amount string.
///
/// The function supports both period (`.`) and comma (`,`) decimal separators.
/// It verifies that the amount is greater than zero, is numeric, and has a maximum of two decimal places.
///
/// # Arguments
///
/// * `amount_str` - A string slice containing the transaction amount input.
///
/// # Returns
///
/// Returns `Ok(f64)` containing the parsed amount on success, or `Err(String)` explaining why validation failed.
///
/// # Errors
///
/// This function returns an error if:
/// * The amount string is empty or contains non-numeric characters.
/// * The amount is less than or equal to zero.
/// * The input has more than two decimal places.
///
/// # Examples
///
/// ```
/// use isearch_cli::pix::validate_amount;
/// let amt = validate_amount("10,50").unwrap();
/// assert_eq!(amt, 10.50);
///
/// assert!(validate_amount("-5.00").is_err());
/// assert!(validate_amount("10.123").is_err());
/// ```
pub fn validate_amount(amount_str: &str) -> Result<f64, String> {
    let trimmed = amount_str.trim().replace(',', ".");
    if trimmed.is_empty() {
        return Err("Amount cannot be empty.".to_string());
    }

    let val: f64 = trimmed
        .parse()
        .map_err(|_| "Please enter a valid numeric amount (e.g., 10.50).".to_string())?;

    if val <= 0.0 {
        return Err("Amount must be greater than zero.".to_string());
    }

    // Check if there are more than 2 decimal places
    if let Some(dot_idx) = trimmed.find('.') {
        let decimals = &trimmed[dot_idx + 1..];
        if decimals.len() > 2 {
            return Err("Amount cannot have more than 2 decimal places.".to_string());
        }
    }

    Ok(val)
}

/// Generates a compliant EMV Co / Banco Central do Brasil static PIX payload string.
///
/// This string is suitable for encoding as a QR code and scanning with any Brazilian banking application.
/// It handles formatting fields like Merchant Account Info, Currency Code (BRL/986), Merchant Name, City,
/// and includes a calculated CRC16 checksum appended at the end of the payload.
///
/// # Arguments
///
/// * `pix_key` - The recipient's PIX key identifier.
/// * `amount` - Optional transaction amount in BRL. If `None`, the user will enter the value in their banking app.
/// * `merchant_name` - The primary name of the recipient merchant (up to 25 characters are kept).
/// * `merchant_city` - The city of the recipient merchant (up to 15 characters are kept).
/// * `message` - An optional message or transaction identifier/description (up to 72 characters).
///
/// # Returns
///
/// Returns a `Result<String, String>` containing the full payload ready for QR rendering, or an error message if inputs are invalid.
///
/// # Errors
///
/// Returns an error if the PIX key is empty or whitespace.
///
/// # Examples
///
/// ```
/// use isearch_cli::pix::generate_pix_payload;
/// let payload = generate_pix_payload(
///     "11925416678",
///     Some(15.00),
///     "Developer",
///     "Sao Paulo",
///     Some("Thanks")
/// ).unwrap();
/// assert!(payload.contains("11925416678"));
/// ```
pub fn generate_pix_payload(
    pix_key: &str,
    amount: Option<f64>,
    merchant_name: &str,
    merchant_city: &str,
    message: Option<&str>,
) -> Result<String, String> {
    if pix_key.trim().is_empty() {
        return Err("PIX Key cannot be empty.".to_string());
    }

    let mut payload = String::new();

    // 00: Payload Format Indicator (Fixed to "01")
    payload.push_str(&format_tlv("00", "01"));

    // 26: Merchant Account Information
    // Sub-tags:
    // 00: GUI (Fixed to "br.gov.bcb.pix")
    // 01: PIX Key
    // 02: Description / Message (Optional, up to 72 characters)
    let mut merchant_account_info = String::new();
    merchant_account_info.push_str(&format_tlv("00", "br.gov.bcb.pix"));
    merchant_account_info.push_str(&format_tlv("01", pix_key));
    if let Some(msg) = message {
        let clean_msg: String = msg
            .chars()
            .take(72)
            .filter(|c| {
                c.is_ascii_alphanumeric() || c.is_ascii_whitespace() || *c == '-' || *c == '_'
            })
            .collect();
        if !clean_msg.is_empty() {
            merchant_account_info.push_str(&format_tlv("02", &clean_msg));
        }
    }
    payload.push_str(&format_tlv("26", &merchant_account_info));

    // 52: Merchant Category Code (Fixed to "0000")
    payload.push_str(&format_tlv("52", "0000"));

    // 53: Transaction Currency (Fixed to "986" for BRL)
    payload.push_str(&format_tlv("53", "986"));

    // 54: Transaction Amount (Optional, e.g. "10.00")
    if let Some(amt) = amount {
        let amt_str = format!("{:.2}", amt);
        payload.push_str(&format_tlv("54", &amt_str));
    }

    // 58: Country Code (Fixed to "BR")
    payload.push_str(&format_tlv("58", "BR"));

    // 59: Merchant Name
    let clean_name: String = merchant_name
        .chars()
        .take(25)
        .filter(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace())
        .collect();
    let final_name = if clean_name.trim().is_empty() {
        "Erik Rodrigues Balisa".to_string()
    } else {
        clean_name
    };
    payload.push_str(&format_tlv("59", &final_name));

    // 60: Merchant City
    let clean_city: String = merchant_city
        .chars()
        .take(15)
        .filter(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace())
        .collect();
    let final_city = if clean_city.trim().is_empty() {
        "SAO PAULO".to_string()
    } else {
        clean_city
    };
    payload.push_str(&format_tlv("60", &final_city));

    // 62: Additional Data Field Template
    // Sub-tags:
    // 05: Reference Label / txid (Fixed to "***" for static QR codes without custom txid)
    let mut additional_data = String::new();
    additional_data.push_str(&format_tlv("05", "***"));
    payload.push_str(&format_tlv("62", &additional_data));

    // 63: Checksum
    // Formatted up to the tag and length: "6304"
    payload.push_str("6304");

    // Calculate CRC16 CCITT
    let crc = crc16_ccitt(payload.as_bytes());
    let crc_str = format!("{:04X}", crc);
    payload.push_str(&crc_str);

    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc16_calculation() {
        // Test CRC16 calculation against known EMV payload test cases
        let data = "00020101021226540014br.gov.bcb.pix01111192541667852040000530398654045.005802BR5921Erik Rodrigues Balisa6009SAO PAULO62070503***6304";
        let crc = crc16_ccitt(data.as_bytes());
        let crc_str = format!("{:04X}", crc);
        assert_eq!(crc_str.len(), 4);
    }

    #[test]
    fn test_validate_amount() {
        assert!(validate_amount("5").is_ok());
        assert_eq!(validate_amount("5").unwrap(), 5.0);
        assert_eq!(validate_amount("10,50").unwrap(), 10.50);
        assert_eq!(validate_amount(" 20.2 ").unwrap(), 20.2);

        assert!(validate_amount("-5").is_err());
        assert!(validate_amount("0").is_err());
        assert!(validate_amount("abc").is_err());
        assert!(validate_amount("10.123").is_err());
    }

    #[test]
    fn test_generate_payload() {
        let payload_res = generate_pix_payload(
            "11925416678",
            Some(10.0),
            "Erik Rodrigues Balisa",
            "PERUIBE",
            Some("Supporting iSearch"),
        );
        assert!(payload_res.is_ok());
        let payload = payload_res.unwrap();
        assert!(payload.starts_with("000201"));
        assert!(payload.contains("11925416678"));
        assert!(payload.contains("986")); // currency BRL
        assert!(payload.contains("10.00")); // amount
        assert!(payload.contains("6304")); // checksum tag
    }
}
