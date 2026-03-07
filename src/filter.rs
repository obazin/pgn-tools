use rayon::prelude::*;

use crate::date::date_to_timestamp;
use crate::PgnGame;

/// Comparison operator for date filters.
#[derive(Debug, PartialEq)]
pub(crate) enum Comparator {
    Lt,
    Le,
    Gt,
    Ge,
}

/// A parsed date filter with an optional comparator and date components.
#[derive(Debug)]
pub struct DateFilter {
    pub(crate) comparator: Option<Comparator>,
    pub(crate) year: i32,
    pub(crate) month: Option<i32>,
    pub(crate) day: Option<i32>,
    /// Precomputed timestamp for comparison filters (missing components default to 1).
    pub(crate) timestamp: i64,
}

/// Parse a filter expression like "2026", "< 2025", or ">= 2025-06"
/// into a `DateFilter`. Supports `<`, `<=`, `>`, `>=` prefixes and
/// year, year-month, or year-month-day date formats.
pub fn parse_filter(expr: &str) -> DateFilter {
    let expr = expr.trim();

    let (comparator, rest) = if let Some(rest) = expr.strip_prefix(">=") {
        (Some(Comparator::Ge), rest.trim())
    } else if let Some(rest) = expr.strip_prefix("<=") {
        (Some(Comparator::Le), rest.trim())
    } else if let Some(rest) = expr.strip_prefix('>') {
        (Some(Comparator::Gt), rest.trim())
    } else if let Some(rest) = expr.strip_prefix('<') {
        (Some(Comparator::Lt), rest.trim())
    } else {
        (None, expr)
    };

    let parts: Vec<&str> = rest.split('-').collect();
    let year: i32 = parts[0].parse().unwrap_or(1970);
    let month: Option<i32> = parts.get(1).map(|s| s.parse().unwrap_or(1));
    let day: Option<i32> = parts.get(2).map(|s| s.parse().unwrap_or(1));
    let timestamp = date_to_timestamp(year, month.unwrap_or(1), day.unwrap_or(1));

    DateFilter {
        comparator,
        year,
        month,
        day,
        timestamp,
    }
}

/// Filter games by a date expression string using parallel iteration.
/// Parses the expression and returns references to matching games.
pub fn filter_games<'a>(games: &'a [PgnGame], expr: &str) -> Vec<&'a PgnGame> {
    let filter = parse_filter(expr);
    games.par_iter().filter(|g| matches_filter(g, &filter)).collect()
}

/// Remove games that don't match the filter expression, collecting
/// matches in parallel. Returns the number of games before filtering.
pub fn retain_matching(games: &mut Vec<PgnGame>, expr: &str) -> usize {
    let total = games.len();
    let filter = parse_filter(expr);
    let filtered = std::mem::take(games)
        .into_par_iter()
        .filter(|g| matches_filter(g, &filter))
        .collect();
    *games = filtered;
    total
}

/// Check whether a game matches a date filter.
///
/// For exact filters (no comparator), date components with "??" are
/// normalized to "01" before comparing year, month, and day.
/// For comparison filters, timestamps are compared directly.
pub fn matches_filter(game: &PgnGame, filter: &DateFilter) -> bool {
    match &filter.comparator {
        None => matches_exact(game, filter),
        Some(cmp) => match cmp {
            Comparator::Lt => game.timestamp < filter.timestamp,
            Comparator::Le => game.timestamp <= filter.timestamp,
            Comparator::Gt => game.timestamp > filter.timestamp,
            Comparator::Ge => game.timestamp >= filter.timestamp,
        },
    }
}

/// Check whether a game matches an exact date filter (no comparator).
/// Unknown date components ("??") are treated as "01".
fn matches_exact(game: &PgnGame, filter: &DateFilter) -> bool {
    let normalized = game.date_raw.replace("??", "01");
    let parts: Vec<&str> = normalized.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    let game_year: i32 = parts[0].parse().unwrap_or(0);
    let game_month: i32 = parts[1].parse().unwrap_or(0);
    let game_day: i32 = parts[2].parse().unwrap_or(0);

    if game_year != filter.year {
        return false;
    }
    if let Some(m) = filter.month {
        if game_month != m {
            return false;
        }
    }
    if let Some(d) = filter.day {
        if game_day != d {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::pgn_date_to_timestamp;

    fn make_game(date_raw: &str) -> PgnGame {
        PgnGame {
            raw: String::new(),
            date_raw: date_raw.to_string(),
            timestamp: pgn_date_to_timestamp(date_raw),
        }
    }

    #[test]
    fn test_parse_filter() {
        let f = parse_filter("2026");
        assert_eq!(f.comparator, None);
        assert_eq!(f.year, 2026);
        assert_eq!(f.month, None);
        assert_eq!(f.day, None);

        let f = parse_filter("2026-01");
        assert_eq!(f.comparator, None);
        assert_eq!(f.year, 2026);
        assert_eq!(f.month, Some(1));
        assert_eq!(f.day, None);

        let f = parse_filter("2026-01-15");
        assert_eq!(f.comparator, None);
        assert_eq!(f.year, 2026);
        assert_eq!(f.month, Some(1));
        assert_eq!(f.day, Some(15));

        let f = parse_filter("< 2025");
        assert_eq!(f.comparator, Some(Comparator::Lt));
        assert_eq!(f.year, 2025);

        let f = parse_filter(">= 2025-06");
        assert_eq!(f.comparator, Some(Comparator::Ge));
        assert_eq!(f.year, 2025);
        assert_eq!(f.month, Some(6));

        let f = parse_filter("<= 2025-06-15");
        assert_eq!(f.comparator, Some(Comparator::Le));
        assert_eq!(f.year, 2025);
        assert_eq!(f.month, Some(6));
        assert_eq!(f.day, Some(15));

        let f = parse_filter("> 2020");
        assert_eq!(f.comparator, Some(Comparator::Gt));
        assert_eq!(f.year, 2020);
    }

    #[test]
    fn test_matches_filter_exact_year() {
        let filter = parse_filter("2025");

        assert!(matches_filter(&make_game("2025.??.??"), &filter));
        assert!(matches_filter(&make_game("2025.06.15"), &filter));
        assert!(matches_filter(&make_game("2025.12.??"), &filter));
        assert!(!matches_filter(&make_game("2024.12.31"), &filter));
        assert!(!matches_filter(&make_game("2026.01.01"), &filter));
    }

    #[test]
    fn test_matches_filter_exact_year_month() {
        let filter = parse_filter("2025-06");

        assert!(matches_filter(&make_game("2025.06.??"), &filter));
        assert!(matches_filter(&make_game("2025.06.15"), &filter));
        assert!(!matches_filter(&make_game("2025.??.??"), &filter)); // ?? -> 01 != 06
        assert!(!matches_filter(&make_game("2025.07.01"), &filter));
        assert!(!matches_filter(&make_game("2024.06.15"), &filter));
    }

    #[test]
    fn test_matches_filter_exact_year_month_matches_unknown() {
        // "2025-01" should match "2025.??.??" because ?? -> 01
        let filter = parse_filter("2025-01");
        assert!(matches_filter(&make_game("2025.??.??"), &filter));
    }

    #[test]
    fn test_matches_filter_exact_full_date() {
        let filter = parse_filter("2025-06-15");

        assert!(matches_filter(&make_game("2025.06.15"), &filter));
        assert!(!matches_filter(&make_game("2025.06.16"), &filter));
        assert!(!matches_filter(&make_game("2025.06.??"), &filter)); // ?? -> 01 != 15
        assert!(!matches_filter(&make_game("2025.??.??"), &filter));
    }

    #[test]
    fn test_matches_filter_exact_full_date_matches_unknown() {
        // "2025-01-01" should match "2025.??.??" because ?? -> 01
        let filter = parse_filter("2025-01-01");
        assert!(matches_filter(&make_game("2025.??.??"), &filter));
        assert!(matches_filter(&make_game("2025.01.??"), &filter));
    }

    #[test]
    fn test_matches_filter_comparators() {
        // < 2025 means before 2025.01.01
        let filter = parse_filter("< 2025");
        assert!(matches_filter(&make_game("2024.12.31"), &filter));
        assert!(matches_filter(&make_game("2020.01.01"), &filter));
        assert!(!matches_filter(&make_game("2025.01.01"), &filter));
        assert!(!matches_filter(&make_game("2025.06.15"), &filter));

        // >= 2025-06 means >= 2025.06.01
        let filter = parse_filter(">= 2025-06");
        assert!(matches_filter(&make_game("2025.06.01"), &filter));
        assert!(matches_filter(&make_game("2025.06.15"), &filter));
        assert!(matches_filter(&make_game("2025.12.31"), &filter));
        assert!(matches_filter(&make_game("2026.01.01"), &filter));
        assert!(!matches_filter(&make_game("2025.05.31"), &filter));
        assert!(!matches_filter(&make_game("2024.12.31"), &filter));

        // > 2025 means > 2025.01.01
        let filter = parse_filter("> 2025");
        assert!(matches_filter(&make_game("2025.01.02"), &filter));
        assert!(matches_filter(&make_game("2026.01.01"), &filter));
        assert!(!matches_filter(&make_game("2025.01.01"), &filter));
        assert!(!matches_filter(&make_game("2024.12.31"), &filter));

        // <= 2025-06-15
        let filter = parse_filter("<= 2025-06-15");
        assert!(matches_filter(&make_game("2025.06.15"), &filter));
        assert!(matches_filter(&make_game("2025.06.14"), &filter));
        assert!(matches_filter(&make_game("2024.01.01"), &filter));
        assert!(!matches_filter(&make_game("2025.06.16"), &filter));
    }

    #[test]
    fn test_filter_with_unknown_date_components() {
        // Comparisons use sort key (??->01)
        let filter = parse_filter("< 2025");
        assert!(matches_filter(&make_game("1988.??.??"), &filter));

        let filter = parse_filter(">= 2025");
        assert!(matches_filter(&make_game("2025.??.??"), &filter));

        // Exact match: ?? is replaced by 01
        let filter = parse_filter("2025-06");
        assert!(!matches_filter(&make_game("2025.??.??"), &filter)); // 01 != 06

        let filter = parse_filter("2025-01");
        assert!(matches_filter(&make_game("2025.??.??"), &filter)); // 01 == 01

        let filter = parse_filter("2025-06-15");
        assert!(!matches_filter(&make_game("2025.06.??"), &filter)); // 01 != 15
        assert!(!matches_filter(&make_game("2025.??.??"), &filter)); // 01 != 06
    }
}
