use std::fs;
use std::process;

use crate::PgnGame;

/// Check if the byte at `pos` is part of a valid UTF-8 sequence.
/// Scans a small window around `pos` as a best-effort heuristic.
fn is_valid_utf8_at(bytes: &[u8], pos: usize) -> bool {
    let start = pos.saturating_sub(3);
    for s in start..=pos {
        let end = (s + 4).min(bytes.len());
        if std::str::from_utf8(&bytes[s..end]).is_ok() {
            return true;
        }
    }
    false
}

/// Count and log invalid UTF-8 bytes found in `raw_bytes`.
fn warn_invalid_utf8(path: &str, raw_bytes: &[u8]) {
    if String::from_utf8(raw_bytes.to_vec()).is_ok() {
        return;
    }
    let count = raw_bytes
        .iter()
        .enumerate()
        .filter(|(i, b)| **b > 127 && !is_valid_utf8_at(raw_bytes, *i))
        .count();
    eprintln!("Warning: '{path}' contains ~{count} invalid UTF-8 byte(s), replacing with \u{FFFD}");
}

/// Read a PGN file, handling invalid UTF-8 and BOM stripping.
///
/// Returns `(content, trailing_newlines)` where `content` has invalid
/// UTF-8 replaced with U+FFFD and the BOM stripped if present.
/// Exits with code 1 on read failure.
pub fn read_pgn(path: &str) -> (String, usize) {
    let raw_bytes = fs::read(path).unwrap_or_else(|e| {
        eprintln!("Error reading '{path}': {e}");
        process::exit(1);
    });

    warn_invalid_utf8(path, &raw_bytes);

    let content = String::from_utf8_lossy(&raw_bytes);
    let content = content.strip_prefix('\u{feff}').unwrap_or(&content).to_string();
    let trailing_newlines = content.len() - content.trim_end_matches('\n').len();

    (content, trailing_newlines)
}

/// Format a list of games into a single string, separated by double
/// newlines, preserving the original trailing newline count (minimum 1).
pub fn format_games(games: &[&PgnGame], trailing_newlines: usize) -> String {
    let mut output = String::new();
    for (i, game) in games.iter().enumerate() {
        output.push_str(&game.raw);
        if i < games.len() - 1 {
            output.push_str("\n\n");
        }
    }
    let n = trailing_newlines.max(1);
    for _ in 0..n {
        output.push('\n');
    }
    output
}
