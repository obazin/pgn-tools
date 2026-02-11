use std::env;
use std::fs;
use std::process;

/// A PGN game represented as its raw text block and a sortable date key.
struct PgnGame {
    /// The full original text of the game (headers + moves), untouched.
    raw: String,
    /// The Date tag value as-is, e.g. "1988.??.??" or "2013.07.27".
    date_raw: String,
    /// A sort key where "??" components are replaced by "01",
    /// yielding a string like "1988.01.01" that sorts lexicographically.
    date_sort_key: String,
}

/// Parse raw Date tag value (e.g. "1988.??.??") into a sortable key
/// by replacing "??" with "01".
fn make_sort_key(date: &str) -> String {
    date.replace("??", "01")
}

/// Extract the value of `[Date "..."]` from a block of PGN header lines.
/// Returns None if no Date tag is found.
fn extract_date(headers: &str) -> Option<String> {
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

/// Split PGN file content into individual game blocks.
///
/// Strategy: a new game starts when we encounter a line starting with `[`
/// and the previous line was blank (or we're at the start of the file).
/// This correctly handles multi-line move text and multiple header tags.
fn parse_games(content: &str) -> Vec<PgnGame> {
    let mut games: Vec<PgnGame> = Vec::new();
    let mut current_lines: Vec<&str> = Vec::new();
    let mut prev_blank = true; // treat start-of-file as "after blank"

    for line in content.lines() {
        let is_tag_line = line.starts_with('[');

        // New game boundary: a tag line preceded by a blank line,
        // and we already have accumulated content from a previous game.
        if is_tag_line && prev_blank && !current_lines.is_empty() {
            // Flush the current game
            let raw = current_lines.join("\n").trim().to_string();
            if !raw.is_empty() {
                let date_raw = extract_date(&raw).unwrap_or_else(|| "????.??.??".to_string());
                let date_sort_key = make_sort_key(&date_raw);
                games.push(PgnGame {
                    raw,
                    date_raw,
                    date_sort_key,
                });
            }
            current_lines = Vec::new();
        }

        current_lines.push(line);
        prev_blank = line.trim().is_empty();
    }

    // Flush the last game
    let raw = current_lines.join("\n").trim().to_string();
    if !raw.is_empty() {
        let date_raw = extract_date(&raw).unwrap_or_else(|| "????.??.??".to_string());
        let date_sort_key = make_sort_key(&date_raw);
        games.push(PgnGame {
            raw,
            date_raw,
            date_sort_key,
        });
    }

    games
}

fn print_usage(program: &str) {
    eprintln!("Usage: {} <input.pgn> <output.pgn> [--desc|--asc]", program);
    eprintln!();
    eprintln!("Sort PGN games by their [Date] tag.");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --asc    Sort oldest first (default)");
    eprintln!("  --desc   Sort newest first");
    eprintln!();
    eprintln!("Date components of \"??\" are treated as \"01\" for sorting.");
    eprintln!("The original date values are preserved in the output file.");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let descending = args.iter().any(|a| a == "--desc");

    // Read input as raw bytes, handling potential invalid UTF-8 gracefully
    let raw_bytes = fs::read(input_path).unwrap_or_else(|e| {
        eprintln!("Error reading '{}': {}", input_path, e);
        process::exit(1);
    });

    if String::from_utf8(raw_bytes.clone()).is_err() {
        let invalid_count = raw_bytes
            .iter()
            .enumerate()
            .filter(|(i, b)| {
                // Quick heuristic: bytes that aren't valid as single-byte UTF-8
                // and aren't valid continuation/leading bytes in context
                **b > 127 && !is_valid_utf8_at(&raw_bytes, *i)
            })
            .count();
        eprintln!(
            "Warning: '{}' contains ~{} invalid UTF-8 byte(s), replacing with \u{FFFD}",
            input_path, invalid_count
        );
    }

    let content = String::from_utf8_lossy(&raw_bytes);
    let content = content.strip_prefix('\u{feff}').unwrap_or(&content);

    // Parse games
    let mut games = parse_games(content);

    let total = games.len();
    eprintln!("Parsed {} games from '{}'", total, input_path);

    // Sort by date_sort_key
    if descending {
        games.sort_by(|a, b| b.date_sort_key.cmp(&a.date_sort_key));
        eprintln!("Sorted DESC (newest first)");
    } else {
        games.sort_by(|a, b| a.date_sort_key.cmp(&b.date_sort_key));
        eprintln!("Sorted ASC (oldest first)");
    }

    // Preview first and last dates
    if let (Some(first), Some(last)) = (games.first(), games.last()) {
        eprintln!("Date range: {} .. {}", first.date_raw, last.date_raw);
    }

    // Count trailing newlines in the original content to preserve them
    let trailing_newlines = content.len() - content.trim_end_matches('\n').len();

    // Write output: games separated by double newline, then preserve
    // the original trailing whitespace.
    let mut output = String::new();
    for (i, game) in games.iter().enumerate() {
        output.push_str(&game.raw);
        if i < total - 1 {
            output.push_str("\n\n");
        }
    }
    // Append original trailing newlines (at least 1)
    let newlines_to_add = if trailing_newlines > 0 {
        trailing_newlines
    } else {
        1
    };
    for _ in 0..newlines_to_add {
        output.push('\n');
    }

    fs::write(output_path, &output).unwrap_or_else(|e| {
        eprintln!("Error writing '{}': {}", output_path, e);
        process::exit(1);
    });

    eprintln!("Written to '{}'", output_path);
}

/// Check if the byte at position `pos` is part of a valid UTF-8 sequence.
/// Used for the diagnostic warning to estimate the number of bad bytes.
fn is_valid_utf8_at(bytes: &[u8], pos: usize) -> bool {
    // Try to find a valid UTF-8 sequence starting at or before `pos`
    // by checking a small window. This is a best-effort heuristic.
    let start = if pos >= 3 { pos - 3 } else { 0 };
    for s in start..=pos {
        let end = (s + 4).min(bytes.len());
        if std::str::from_utf8(&bytes[s..end]).is_ok() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_sort_key() {
        assert_eq!(make_sort_key("1988.??.??"), "1988.01.01");
        assert_eq!(make_sort_key("2002.06.??"), "2002.06.01");
        assert_eq!(make_sort_key("2013.07.27"), "2013.07.27");
    }

    #[test]
    fn test_extract_date() {
        let headers = r#"[Event "Test"]
[Site "Nowhere"]
[Date "1988.??.??"]
[Round "1"]"#;
        assert_eq!(extract_date(headers), Some("1988.??.??".to_string()));
    }

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
        assert_eq!(games[0].date_sort_key, "2020.05.10");
        assert_eq!(games[1].date_raw, "1988.??.??");
        assert_eq!(games[1].date_sort_key, "1988.01.01");
    }

    #[test]
    fn test_sorting_asc() {
        let pgn = r#"[Event "C"]
[Date "2020.05.10"]
[Result "1-0"]

1. e4 e5 1-0

[Event "A"]
[Date "1988.??.??"]
[Result "0-1"]

1. d4 d5 0-1

[Event "B"]
[Date "2002.06.??"]
[Result "1/2-1/2"]

1. c4 e5 1/2-1/2
"#;
        let mut games = parse_games(pgn);
        games.sort_by(|a, b| a.date_sort_key.cmp(&b.date_sort_key));
        assert_eq!(games[0].date_raw, "1988.??.??");
        assert_eq!(games[1].date_raw, "2002.06.??");
        assert_eq!(games[2].date_raw, "2020.05.10");
    }
}
