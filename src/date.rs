/// Check whether a year is a leap year (Gregorian calendar).
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Cumulative days before each month (non-leap year, 1-indexed).
const DAYS_BEFORE_MONTH: [i64; 13] = [0, 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

/// Total days from year 1 to the start of the given year.
fn days_before_year(year: i32) -> i64 {
    let y = (year - 1) as i64;
    365 * y + y / 4 - y / 100 + y / 400
}

/// Convert a year/month/day to seconds since Unix epoch (1970-01-01).
/// Returns a negative value for dates before 1970-01-01.
pub fn date_to_timestamp(year: i32, month: i32, day: i32) -> i64 {
    let mut total = days_before_year(year) + DAYS_BEFORE_MONTH[month as usize] + day as i64 - 1;
    if month > 2 && is_leap_year(year) {
        total += 1;
    }
    (total - days_before_year(1970)) * 86400
}

/// Parse a PGN date string like "2025.06.15" or "1988.??.??" into a
/// Unix timestamp. Unknown components ("??") are treated as "01".
pub(crate) fn pgn_date_to_timestamp(pgn_date: &str) -> i64 {
    let normalized = pgn_date.replace("??", "01");
    let parts: Vec<&str> = normalized.split('.').collect();
    let year = parts.first().and_then(|s| s.parse::<i32>().ok()).unwrap_or(1970);
    let month = parts.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
    let day = parts.get(2).and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
    date_to_timestamp(year, month, day)
}

/// Extract the value of `[Date "..."]` from a block of PGN header lines.
/// Returns `None` if no Date tag is found.
pub(crate) fn extract_date(headers: &str) -> Option<String> {
    for line in headers.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[Date ") {
            if let (Some(start), Some(end)) = (trimmed.find('"'), trimmed.rfind('"')) {
                if start < end {
                    return Some(trimmed[start + 1..end].to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_to_timestamp() {
        assert_eq!(date_to_timestamp(1970, 1, 1), 0);
        assert_eq!(date_to_timestamp(1970, 1, 2), 86400);
        assert_eq!(date_to_timestamp(2000, 1, 1), 946684800);
        assert!(date_to_timestamp(1969, 12, 31) < 0);
        assert_eq!(date_to_timestamp(1969, 12, 31), -86400);
    }

    #[test]
    fn test_pgn_date_to_timestamp() {
        assert_eq!(pgn_date_to_timestamp("1970.01.01"), 0);
        assert_eq!(pgn_date_to_timestamp("2000.01.01"), 946684800);
        assert_eq!(pgn_date_to_timestamp("1988.??.??"), date_to_timestamp(1988, 1, 1));
        assert_eq!(pgn_date_to_timestamp("2002.06.??"), date_to_timestamp(2002, 6, 1));
        assert_eq!(pgn_date_to_timestamp("2013.07.27"), date_to_timestamp(2013, 7, 27));
    }

    #[test]
    fn test_extract_date() {
        let headers = r#"[Event "Test"]
[Site "Nowhere"]
[Date "1988.??.??"]
[Round "1"]"#;
        assert_eq!(extract_date(headers), Some("1988.??.??".to_string()));
    }
}
