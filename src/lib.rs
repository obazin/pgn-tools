pub mod date;
pub mod filter;
pub mod sort;
pub mod util;

use date::*;

/// A PGN game represented as its raw text block and a sortable timestamp.
pub struct PgnGame {
    /// The full original text of the game (headers + moves), untouched.
    pub raw: String,
    /// The Date tag value as-is, e.g. "1988.??.??" or "2013.07.27".
    pub date_raw: String,
    /// Unix timestamp (seconds since 1970-01-01) for sorting/comparison.
    /// Can be negative for pre-1970 dates.
    pub timestamp: i64,
}

/// Build a `PgnGame` from accumulated raw lines.
/// Returns `None` if the text is empty after trimming.
fn flush_game(lines: &[&str]) -> Option<PgnGame> {
    let raw = lines.join("\n").trim().to_string();
    if raw.is_empty() {
        return None;
    }
    let date_raw = extract_date(&raw).unwrap_or_else(|| "????.??.??".to_string());
    let timestamp = pgn_date_to_timestamp(&date_raw);
    Some(PgnGame {
        raw,
        date_raw,
        timestamp,
    })
}

/// Split PGN file content into individual game blocks.
///
/// A new game boundary is detected when a line starting with `[` follows
/// a blank line (or the start of the file). This correctly handles
/// multi-line move text and multiple header tags.
pub fn parse_games(content: &str) -> Vec<PgnGame> {
    let mut games: Vec<PgnGame> = Vec::new();
    let mut current_lines: Vec<&str> = Vec::new();
    let mut prev_blank = true;

    for line in content.lines() {
        if line.starts_with('[') && prev_blank && !current_lines.is_empty() {
            if let Some(game) = flush_game(&current_lines) {
                games.push(game);
            }
            current_lines = Vec::new();
        }

        current_lines.push(line);
        prev_blank = line.trim().is_empty();
    }

    if let Some(game) = flush_game(&current_lines) {
        games.push(game);
    }

    games
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_two_games() {
        let pgn = r#"[Event "A"]
[Date "2020.05.10"]
[Result "1-0"]

1. e4 e5 1-0

[Event "B"]
[Date "1988.??.??"]
[Result "0-1"]

1. d4 d5 0-1
"#;
        let games = parse_games(pgn);
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].date_raw, "2020.05.10");
        assert_eq!(games[0].timestamp, date_to_timestamp(2020, 5, 10));
        assert_eq!(games[1].date_raw, "1988.??.??");
        assert_eq!(games[1].timestamp, date_to_timestamp(1988, 1, 1));
    }
}
