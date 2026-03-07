use pgn_tools::date::*;
use pgn_tools::filter::*;
use pgn_tools::{parse_games, PgnGame};

fn load_jones() -> Vec<PgnGame> {
    let content = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/jones.pgn")
    ).expect("tests/jones.pgn must exist");
    parse_games(&content)
}

#[test]
fn jones_parse_total_games() {
    let games = load_jones();
    assert_eq!(games.len(), 1484);
}

#[test]
fn jones_sort_asc() {
    let mut games = load_jones();
    games.sort_by_key(|g| g.timestamp);

    // Earliest games have unknown dates from 1997 (??->01 → 1997.01.01)
    assert!(games[0].date_raw.starts_with("1997"));
    // Latest game is 2021.07.04
    assert_eq!(games.last().unwrap().date_raw, "2021.07.04");

    // Verify monotonically non-decreasing timestamps
    for w in games.windows(2) {
        assert!(w[0].timestamp <= w[1].timestamp,
            "Out of order: {} ({}) > {} ({})",
            w[0].date_raw, w[0].timestamp, w[1].date_raw, w[1].timestamp);
    }
}

#[test]
fn jones_sort_desc() {
    let mut games = load_jones();
    games.sort_by_key(|g| std::cmp::Reverse(g.timestamp));

    assert_eq!(games[0].date_raw, "2021.07.04");
    assert!(games.last().unwrap().date_raw.starts_with("1997"));

    // Verify monotonically non-increasing timestamps
    for w in games.windows(2) {
        assert!(w[0].timestamp >= w[1].timestamp);
    }
}

#[test]
fn jones_sort_preserves_all_games() {
    let mut games = load_jones();
    assert_eq!(games.len(), 1484);
    games.sort_by_key(|g| g.timestamp);
    assert_eq!(games.len(), 1484);
    games.sort_by_key(|g| std::cmp::Reverse(g.timestamp));
    assert_eq!(games.len(), 1484);
}

#[test]
fn jones_filter_year_2014() {
    let games = load_jones();
    let filter = parse_filter("2014");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    assert_eq!(filtered.len(), 65);
}

#[test]
fn jones_filter_year_1998() {
    let games = load_jones();
    let filter = parse_filter("1998");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    // 1998: 2 games (1 with ??.?? date, 1 from 1998.12.17)
    assert_eq!(filtered.len(), 2);
}

#[test]
fn jones_filter_year_2021() {
    let games = load_jones();
    let filter = parse_filter("2021");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    assert_eq!(filtered.len(), 42);
}

#[test]
fn jones_filter_year_month_2006_02() {
    let games = load_jones();
    let filter = parse_filter("2006-02");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    assert_eq!(filtered.len(), 6);
}

#[test]
fn jones_filter_year_month_single_game() {
    let games = load_jones();

    // 2014.07 has exactly 1 game
    let filter = parse_filter("2014-07");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    assert_eq!(filtered.len(), 1);

    // 2012.05 has exactly 1 game
    let filter = parse_filter("2012-05");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    assert_eq!(filtered.len(), 1);

    // 2020.03 has exactly 1 game
    let filter = parse_filter("2020-03");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    assert_eq!(filtered.len(), 1);
}

#[test]
fn jones_filter_exact_date() {
    let games = load_jones();

    // The very first game in the file: 2014.08.02
    let filter = parse_filter("2014-08-02");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    assert!(!filtered.is_empty());
    for g in &filtered {
        assert_eq!(g.date_raw, "2014.08.02");
    }
}

#[test]
fn jones_filter_lt_2000() {
    let games = load_jones();
    let filter = parse_filter("< 2000");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    // All filtered games must have timestamp < 2000.01.01
    let cutoff = date_to_timestamp(2000, 1, 1);
    for g in &filtered {
        assert!(g.timestamp < cutoff,
            "Game {} should be before 2000", g.date_raw);
    }
    // 1997 (9) + 1998 (2) + 1999 (16) = 27, but 1997.??.?? → 1997.01.01 < 2000
    // and 1998.??.?? → 1998.01.01 < 2000, 1998.12.17 < 2000
    // 1999 games all < 2000
    assert_eq!(filtered.len(), 27);
}

#[test]
fn jones_filter_ge_2020() {
    let games = load_jones();
    let filter = parse_filter(">= 2020");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    // 2020 (20) + 2021 (42) = 62, but there's a 2019.11.24 game that is < 2020
    let cutoff = date_to_timestamp(2020, 1, 1);
    for g in &filtered {
        assert!(g.timestamp >= cutoff,
            "Game {} should be >= 2020", g.date_raw);
    }
    assert_eq!(filtered.len(), 62);
}

#[test]
fn jones_filter_gt_2021() {
    let games = load_jones();
    let filter = parse_filter("> 2021");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    // > 2021.01.01 means all 2021 games except those on exactly 2021.01.01
    let cutoff = date_to_timestamp(2021, 1, 1);
    for g in &filtered {
        assert!(g.timestamp > cutoff);
    }
}

#[test]
fn jones_filter_le_1999_12_31() {
    let games = load_jones();
    let filter = parse_filter("<= 1999-12-31");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    let cutoff = date_to_timestamp(1999, 12, 31);
    for g in &filtered {
        assert!(g.timestamp <= cutoff);
    }
    // Same set as < 2000 since 1999.12.31 < 2000.01.01
    assert_eq!(filtered.len(), 27);
}

#[test]
fn jones_filter_nonexistent_year() {
    let games = load_jones();
    let filter = parse_filter("1950");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    assert_eq!(filtered.len(), 0);
}

#[test]
fn jones_filter_and_sort_combined() {
    let mut games = load_jones();
    games.sort_by_key(|g| g.timestamp);

    // Filter to 2006 and verify the filtered results are still sorted
    let filter = parse_filter("2006");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    assert_eq!(filtered.len(), 65);

    for w in filtered.windows(2) {
        assert!(w[0].timestamp <= w[1].timestamp);
    }

    // First 2006 game should be from early 2006
    assert!(filtered[0].date_raw.starts_with("2006."));
    assert!(filtered.last().unwrap().date_raw.starts_with("2006."));
}

#[test]
fn jones_filter_desc_sort_then_filter() {
    let mut games = load_jones();
    games.sort_by_key(|g| std::cmp::Reverse(g.timestamp));

    let filter = parse_filter(">= 2019");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();

    // Should be in descending order
    for w in filtered.windows(2) {
        assert!(w[0].timestamp >= w[1].timestamp);
    }

    // 2019 (124) + 2020 (20) + 2021 (42) = 186
    assert_eq!(filtered.len(), 186);
    assert_eq!(filtered[0].date_raw, "2021.07.04"); // newest first
}

#[test]
fn jones_unknown_dates_count() {
    let games = load_jones();
    let unknown: Vec<_> = games.iter()
        .filter(|g| g.date_raw.contains("??"))
        .collect();
    assert_eq!(unknown.len(), 11);
}

#[test]
fn jones_unknown_dates_sort_to_beginning() {
    let mut games = load_jones();
    games.sort_by_key(|g| g.timestamp);

    // The 9 games with "1997.??.??" should sort to the very beginning
    // since 1997.01.01 is the earliest timestamp in the file
    let first_nine: Vec<_> = games[..9].iter().map(|g| &g.date_raw).collect();
    for d in &first_nine {
        assert!(d.starts_with("1997"), "Expected 1997 date, got {d}");
    }
}

#[test]
fn jones_unknown_dates_filter_and_sort() {
    let mut games = load_jones();
    games.sort_by_key(|g| g.timestamp);

    // "1997.??.??" → timestamp 1997.01.01, exact match treats ?? as 01
    // Filtering "1997-01" should match all 9 "1997.??.??" games (??→01)
    let filter = parse_filter("1997-01");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    assert_eq!(filtered.len(), 9);
    for g in &filtered {
        assert_eq!(g.date_raw, "1997.??.??");
    }
    // They should all share the same timestamp and appear sorted together
    let ts = filtered[0].timestamp;
    assert_eq!(ts, date_to_timestamp(1997, 1, 1));
    for g in &filtered {
        assert_eq!(g.timestamp, ts);
    }

    // "1998.??.??" → timestamp 1998.01.01
    // Filtering "1998-12" should NOT match it (??→01 ≠ 12), but should
    // match the real 1998.12.17 game
    let filter = parse_filter("1998-12");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].date_raw, "1998.12.17");

    // Filtering "1998-01" should match only the "1998.??.??" game (??→01)
    let filter = parse_filter("1998-01");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].date_raw, "1998.??.??");

    // "2000.??.??" → timestamp 2000.01.01
    // Filtering "< 2000" excludes it (2000.01.01 is NOT < 2000.01.01)
    let filter = parse_filter("< 2000");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    assert!(filtered.iter().all(|g| g.date_raw != "2000.??.??"));

    // Filtering ">= 2000" includes it
    let filter = parse_filter(">= 2000");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    assert!(filtered.iter().any(|g| g.date_raw == "2000.??.??"));

    // After sorting asc, "2000.??.??" (ts 2000.01.01) should appear
    // before any 2000.01.xx game with xx > 01
    let filter = parse_filter("2000");
    let filtered: Vec<_> = games.iter().filter(|g| matches_filter(g, &filter)).collect();
    let unknown_pos = filtered.iter()
        .position(|g| g.date_raw == "2000.??.??")
        .expect("2000.??.?? should be in year 2000 filter");
    // It sorts to timestamp 2000.01.01, so it should be at the start
    // of the 2000 games (or tied with any real 2000.01.01 game)
    for g in &filtered[..unknown_pos] {
        assert!(g.timestamp <= date_to_timestamp(2000, 1, 1),
            "Game {} at position before 2000.??.?? should have timestamp <= 2000.01.01",
            g.date_raw);
    }
}
