use std::fs;
use std::process;

use clap::{CommandFactory, Parser};

use pgn_tools::filter::retain_matching;
use pgn_tools::sort::{sort_asc, sort_desc};
use pgn_tools::util;
use pgn_tools::util::format_games;
use pgn_tools::{parse_games, PgnGame};

#[derive(Parser)]
#[command(about = "Sort and/or filter PGN games by their [Date] tag.")]
#[command(after_help = "\
Filter expressions:
  2026          Games from year 2026
  2026-01       Games from January 2026
  2026-01-15    Games from exactly Jan 15, 2026
  < 2025        Games before 2025
  >= 2025-06    Games from June 2025 onwards

Date components of \"??\" are treated as \"01\" for sorting.
The original date values are preserved in the output.")]
struct Cli {
    /// Input PGN file
    #[arg(required_unless_present = "completions")]
    input: Option<String>,

    /// Generate shell completions and exit
    #[arg(long, value_name = "SHELL")]
    completions: Option<clap_complete::Shell>,

    /// Output PGN file (defaults to <input>-sorted.pgn, or <input>-filtered.pgn when --filter is used)
    output: Option<String>,

    /// Sort newest first
    #[arg(long)]
    desc: bool,

    /// Sort oldest first (default)
    #[arg(long)]
    asc: bool,

    /// Filter games by date expression
    #[arg(long)]
    filter: Option<String>,
}

/// Validate CLI argument constraints that clap cannot express.
fn validate_args(cli: &Cli) {
    if cli.asc && cli.desc {
        eprintln!("Error: --asc and --desc are mutually exclusive");
        process::exit(1);
    }
}

/// Build a default output path by inserting `suffix` before ".pgn" (or appending it).
fn default_output_path(input: &str, suffix: &str) -> String {
    if let Some(stem) = input.strip_suffix(".pgn") {
        format!("{stem}-{suffix}.pgn")
    } else {
        format!("{input}-{suffix}")
    }
}

/// Determine the output path based on CLI arguments.
fn output_path(cli: &Cli, input: &str) -> String {
    if let Some(ref path) = cli.output {
        path.clone()
    } else if cli.filter.is_some() {
        default_output_path(input, "filtered")
    } else {
        default_output_path(input, "sorted")
    }
}

/// Format and write games to a file. Exits on I/O error.
fn write_output(path: &str, games: &[PgnGame], trailing_newlines: usize) {
    let refs: Vec<&PgnGame> = games.iter().collect();
    let output = format_games(&refs, trailing_newlines);
    if let Err(e) = fs::write(path, &output) {
        eprintln!("Error writing '{path}': {e}");
        process::exit(1);
    }
    eprintln!("Written to '{path}'");
}

/// Sort games in place and log the sort direction.
fn sort_games(games: &mut [PgnGame], descending: bool) {
    if descending {
        sort_desc(games);
        eprintln!("Sorted DESC (newest first)");
    } else {
        sort_asc(games);
        eprintln!("Sorted ASC (oldest first)");
    }
    if let (Some(first), Some(last)) = (games.first(), games.last()) {
        eprintln!("Date range: {} .. {}", first.date_raw, last.date_raw);
    }
}

fn main() {
    let cli = Cli::parse();

    if let Some(shell) = cli.completions {
        clap_complete::generate(
            shell,
            &mut Cli::command(),
            "pgn-tools",
            &mut std::io::stdout(),
        );
        return;
    }

    let input = cli.input.as_deref().unwrap();
    validate_args(&cli);

    let (content, trailing_newlines) = util::read_pgn(input);
    let mut games = parse_games(&content);
    eprintln!("Parsed {} games from '{input}'", games.len());

    // Filter first to reduce the working set before sorting
    if let Some(ref expr) = cli.filter {
        let total = retain_matching(&mut games, expr);
        eprintln!("Filtered {} / {total} games matching '{expr}'", games.len());
        if games.is_empty() {
            eprintln!("No games matched the filter; no output file written");
            return;
        }
    }

    sort_games(&mut games, cli.desc);
    write_output(&output_path(&cli, input), &games, trailing_newlines);
}
