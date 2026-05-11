# tokenburn

Track token usage and burn for Claude Code and Codex.

## Install

```sh
brew install RMahshie/tap/tokenburn
```

Or run from source:

```sh
cargo run -- --live
```

## Usage

One-shot mode prints the current dashboard and exits:

```sh
tokenburn
```

Live mode opens the interactive TUI dashboard:

```sh
tokenburn --live
```

Use a specific range:

```sh
tokenburn --range 24h
tokenburn --range 7d
tokenburn --range 30d
tokenburn --range lifetime
```

`lifetime` means the locally available logs retained on this machine, not true account lifetime usage.

Use a custom inclusive date range:

```sh
tokenburn --from 2026-05-01 --to 2026-05-09
```

Set the live refresh interval in seconds:

```sh
tokenburn --live --interval 15
```

Set Claude Code transcript retention to 10 years:

```sh
tokenburn --fix-claude-retention
```

## Live Keys

- `q` or `Esc`: quit
- `Tab` or arrow keys: switch provider
- `s`: cycle Claude views: main, subagents, combined, usage cache
- `a`: add 10-year Claude transcript retention when prompted
- `r`: cycle range
- `p`: pause or resume refresh
- `?`: show or hide help

## Stats

- `Input`: non-cached prompt/input tokens. For Codex this is `input_tokens - cached_input_tokens`.
- `Output`: non-reasoning response/output tokens. For Codex this is `output_tokens - reasoning_output_tokens`.
- `Cache Read`: cached input tokens reused by the model.
- `Cache Write`: Claude Code cache creation tokens.
- `Cached Input`: Codex cached input tokens.
- `Reasoning`: Codex reasoning output tokens.
- `Total`: sum of all displayed token categories for that tool.
- `Daily Avg`: total divided by the selected range length.
- `% Total`: that metric's share of the tool total.
- `Change vs prior period`: percent change compared with the previous range of the same length.
- `Daily Trend`: recent daily burn pattern for the selected range.
- Claude subagent usage is tracked separately from the main Claude Code totals.
- Claude usage cache mode shows `~/.claude/stats-cache.json` aggregate `/usage` data separately from transcript-derived charts.

## Data Sources

- Claude Code: `~/.claude/projects/**/*.jsonl`
- Claude usage cache: `~/.claude/stats-cache.json`
- Codex: `~/.codex/sessions/<year>/<month>/<day>/*.jsonl`
