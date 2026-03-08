# pgn-tools

A command-line tool to sort and filter [PGN](https://en.wikipedia.org/wiki/Portable_Game_Notation) chess game files by their `[Date]` tag.

## Features

- **Sort games** by date in ascending (oldest first) or descending (newest first) order
- **Filter games** by year, month, exact date, or date range using comparison operators
- **Combine sort and filter** in a single invocation
- **Handles unknown dates** — PGN dates with `??` components (e.g. `1997.??.??`) are treated as `01` for sorting and filtering, while the original values are preserved in the output
- **Tolerates encoding issues** — invalid UTF-8 bytes are replaced with U+FFFD and a warning is printed; BOM is stripped automatically
- **Preserves formatting** — trailing newlines from the original file are maintained in the output

## Installation

```
cargo build --release
```

The binary will be at `target/release/pgn-tools`.

## Usage

```
pgn-tools <INPUT> [OUTPUT] [OPTIONS]
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `INPUT`  | Yes      | Path to the input PGN file |
| `OUTPUT` | No       | Path to the output file (defaults to `<input>-sorted.pgn`, or `<input>-filtered.pgn` when `--filter` is used) |

### Options

| Option | Description |
|--------|-------------|
| `--asc` | Sort oldest first (default) |
| `--desc` | Sort newest first |
| `--filter <EXPR>` | Filter games by date expression |
| `--filter-file <PATH>` | Custom output path for filtered games (default: `<input>-filtered.pgn`) |
| `--completions <SHELL>` | Generate shell completions and exit (`bash`, `zsh`, `fish`, `elvish`, `powershell`) |
| `-h, --help` | Print help with filter expression syntax |

The input file is never modified. When no output file is given, the result is written to `<input>-sorted.pgn` (or `<input>-filtered.pgn` when `--filter` is used).

## Examples

### Sort (writes to `games-sorted.pgn`)

```
pgn-tools games.pgn
```

### Sort to a specific file

```
pgn-tools games.pgn sorted.pgn
```

### Sort newest first

```
pgn-tools games.pgn --desc
```

### Filter games from a specific year

```
pgn-tools games.pgn --filter 2014
```

This writes matching games to `games-filtered.pgn` (default output path).

### Filter games from a specific month

```
pgn-tools games.pgn --filter 2025-06
```

### Filter games from an exact date

```
pgn-tools games.pgn --filter 2025-06-15
```

### Filter with comparison operators

```
# Games before 2000
pgn-tools games.pgn --filter "< 2000"

# Games from June 2025 onwards
pgn-tools games.pgn --filter ">= 2025-06"

# Games after January 1, 2020
pgn-tools games.pgn --filter "> 2020"

# Games up to and including December 31, 1999
pgn-tools games.pgn --filter "<= 1999-12-31"
```

### Custom filter output path

```
pgn-tools games.pgn --filter 2014 --filter-file year2014.pgn
```

### Filter and sort in one command

```
pgn-tools games.pgn --desc --filter ">= 2020"
```

This writes matching games sorted newest-first to `games-filtered.pgn`.

### Custom output path

```
pgn-tools games.pgn recent.pgn --desc --filter ">= 2020"
```

## Filter expressions

| Expression | Matches |
|------------|---------|
| `2026` | All games from year 2026 |
| `2026-01` | Games from January 2026 |
| `2026-01-15` | Games from exactly January 15, 2026 |
| `< 2025` | Games before January 1, 2025 |
| `<= 2025-06-15` | Games on or before June 15, 2025 |
| `> 2020` | Games after January 1, 2020 |
| `>= 2025-06` | Games from June 1, 2025 onwards |

## Handling of unknown dates

PGN files often contain dates with unknown components, such as `1997.??.??` or `2005.06.??`. These are handled as follows:

- **For sorting**: `??` is replaced with `01`, so `1997.??.??` sorts as `1997.01.01`
- **For exact filters**: `??` is replaced with `01` before comparing, so `--filter 1997-01` matches `1997.??.??`
- **In output**: the original date string is always preserved unchanged

## Example output

```
$ pgn-tools jones.pgn --desc --filter ">= 2020"
Parsed 1484 games from 'jones.pgn'
Filtered 62 / 1484 games matching '>= 2020'
Sorted DESC (newest first)
Date range: 2021.07.04 .. 2020.01.18
Written to 'jones-filtered.pgn'
```

## Shell completions

Generate a completion script and place it where your shell can find it:

```
# Zsh
pgn-tools --completions zsh > ~/.zsh/completions/_pgn-tools

# Bash
pgn-tools --completions bash > ~/.local/share/bash-completion/completions/pgn-tools

# Fish
pgn-tools --completions fish > ~/.config/fish/completions/pgn-tools.fish
```

Restart your shell (or `source` the file) to enable tab-completion for all options and arguments.

## Running tests

```
cargo test
```

## License

This project is licensed under the [MIT License](LICENSE).
