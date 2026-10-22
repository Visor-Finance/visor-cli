//! Parsers for command-line values. Each returns the exact string the API
//! expects, so a typo fails here instead of costing a request.

use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone};
use uuid::Uuid;

/// Normalizes an amount to a positive decimal with up to 2 places.
/// Accepts `12.5`, `12.50`, and the Brazilian `12,50` or `1.234,56`.
pub fn amount(value: &str) -> Result<String, String> {
    let value = value.trim().trim_start_matches('+');
    if value.starts_with('-') {
        return Err(format!(
            "`{value}` is negative; pass a positive amount and --type debit or --type credit"
        ));
    }
    let invalid = || format!("`{value}` is not an amount; use a positive number such as 42.90");

    let normalized = if value.contains(',') {
        value.replace('.', "").replace(',', ".")
    } else {
        value.to_owned()
    };

    let (whole, fraction) = normalized.split_once('.').unwrap_or((&normalized, ""));
    let digits = |part: &str| part.chars().all(|c| c.is_ascii_digit());
    if whole.is_empty() || !digits(whole) || !digits(fraction) {
        return Err(invalid());
    }
    if fraction.len() > 2 {
        return Err(format!("`{value}` has more than 2 decimal places"));
    }

    let whole = whole.trim_start_matches('0');
    let whole = if whole.is_empty() { "0" } else { whole };
    let amount = format!("{whole}.{fraction:0<2}");
    if amount == "0.00" {
        return Err("the amount must be greater than zero".to_owned());
    }
    Ok(amount)
}

/// Normalizes a Pix amount like [`amount`]; a Pix only sends money.
pub fn pix_amount(value: &str) -> Result<String, String> {
    if value.trim().starts_with('-') {
        return Err(format!(
            "`{value}` is negative; a Pix amount is positive, such as 20.00"
        ));
    }
    amount(value)
}

/// Validates a `YYYY-MM-DD` date.
pub fn date(value: &str) -> Result<String, String> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|date| date.format("%Y-%m-%d").to_string())
        .map_err(|_| format!("`{value}` is not a date; use YYYY-MM-DD"))
}

/// Turns `now`, a `YYYY-MM-DD` date (noon, local time), or an RFC 3339 time
/// into an RFC 3339 time with an offset.
pub fn datetime(value: &str) -> Result<String, String> {
    datetime_at(value, &Local::now())
}

fn datetime_at<Tz: TimeZone>(value: &str, now: &DateTime<Tz>) -> Result<String, String>
where
    Tz::Offset: std::fmt::Display,
{
    if value.eq_ignore_ascii_case("now") {
        return Ok(now.to_rfc3339_opts(chrono::SecondsFormat::Secs, false));
    }
    if let Ok(time) = DateTime::parse_from_rfc3339(value) {
        return Ok(time.to_rfc3339_opts(chrono::SecondsFormat::Secs, false));
    }
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| format!("`{value}` is not a date; use YYYY-MM-DD, an RFC 3339 time, or `now`"))?;
    // Noon keeps the day the same in every time zone the API might use.
    now.timezone()
        .from_local_datetime(&date.and_time(NaiveTime::from_hms_opt(12, 0, 0).expect("noon is valid")))
        .single()
        .map(|time| time.to_rfc3339_opts(chrono::SecondsFormat::Secs, false))
        .ok_or_else(|| format!("`{value}` is not a valid local time"))
}

/// Validates an ID before it goes into a URL path.
pub fn id(value: &str) -> Result<String, String> {
    Uuid::parse_str(value)
        .map(|id| id.hyphenated().to_string())
        .map_err(|_| format!("`{value}` is not an ID; IDs look like 3f2b1c4e-…"))
}

#[must_use]
pub fn is_id(value: &str) -> bool {
    Uuid::parse_str(value).is_ok()
}

/// Validates an Idempotency-Key: 1 to 128 letters, digits, or `-_.:`.
pub fn idempotency_key(value: &str) -> Result<String, String> {
    let valid = (1..=128).contains(&value.len())
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'));
    if valid {
        Ok(value.to_owned())
    } else {
        Err("an idempotency key is 1 to 128 letters, digits, or - _ . :".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    #[test]
    fn normalizes_amounts() {
        assert_eq!(amount("12").unwrap(), "12.00");
        assert_eq!(amount("12.5").unwrap(), "12.50");
        assert_eq!(amount("12,5").unwrap(), "12.50");
        assert_eq!(amount("1.234,56").unwrap(), "1234.56");
        assert_eq!(amount("+0.99").unwrap(), "0.99");
        assert_eq!(amount("007.10").unwrap(), "7.10");
    }

    #[test]
    fn rejects_bad_amounts() {
        for value in ["", "-5", "abc", "1.234", "0", "0,00", "1e3", ".5", "1.2.3"] {
            assert!(amount(value).is_err(), "{value} should be rejected");
        }
    }

    #[test]
    fn pix_amounts_are_positive() {
        assert_eq!(pix_amount("20,00").unwrap(), "20.00");
        assert!(
            pix_amount("-20")
                .unwrap_err()
                .contains("a Pix amount is positive")
        );
    }

    #[test]
    fn validates_dates() {
        assert_eq!(date("2026-10-01").unwrap(), "2026-10-01");
        assert!(date("2026-13-01").is_err());
        assert!(date("01/10/2026").is_err());
    }

    #[test]
    fn parses_datetimes() {
        let brasilia = FixedOffset::west_opt(3 * 3600).unwrap();
        let now = brasilia.with_ymd_and_hms(2026, 10, 21, 9, 30, 0).unwrap();

        assert_eq!(datetime_at("now", &now).unwrap(), "2026-10-21T09:30:00-03:00");
        assert_eq!(
            datetime_at("2026-10-01", &now).unwrap(),
            "2026-10-01T12:00:00-03:00"
        );
        assert_eq!(
            datetime_at("2026-10-01T08:15:00Z", &now).unwrap(),
            "2026-10-01T08:15:00+00:00"
        );
        assert!(datetime_at("yesterday", &now).is_err());
    }

    #[test]
    fn validates_ids() {
        assert_eq!(
            id("3F2B1C4E-0000-4000-8000-000000000001").unwrap(),
            "3f2b1c4e-0000-4000-8000-000000000001"
        );
        assert!(id("../me").is_err());
    }

    #[test]
    fn validates_idempotency_keys() {
        assert!(idempotency_key("2026-10-01-padaria").is_ok());
        assert!(idempotency_key("row:42").is_ok());
        assert!(idempotency_key("").is_err());
        assert!(idempotency_key("has space").is_err());
        assert!(idempotency_key(&"a".repeat(129)).is_err());
    }
}
