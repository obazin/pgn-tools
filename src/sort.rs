use std::cmp::Reverse;

use crate::PgnGame;

/// Sort games by date, oldest first.
pub fn sort_asc(games: &mut [PgnGame]) {
    games.sort_by_key(|g| g.timestamp);
}

/// Sort games by date, newest first.
pub fn sort_desc(games: &mut [PgnGame]) {
    games.sort_by_key(|g| Reverse(g.timestamp));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_games;

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
        sort_asc(&mut games);
        assert_eq!(games[0].date_raw, "1988.??.??");
        assert_eq!(games[1].date_raw, "2002.06.??");
        assert_eq!(games[2].date_raw, "2020.05.10");
    }
}
