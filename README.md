# tokenburn

Track token usage and burn for Claude Code and Codex.

## Run

One-shot mode prints the current dashboard and exits:

```sh
cargo run
```

Live mode opens the interactive TUI dashboard:

```sh
cargo run -- --live
```

Use a specific range:

```sh
cargo run -- --range 24h
cargo run -- --range 7d
cargo run -- --range 30d
cargo run -- --range lifetime
```

Use a custom inclusive date range:

```sh
cargo run -- --from 2026-05-01 --to 2026-05-09
```

Set the live refresh interval in seconds:

```sh
cargo run -- --live --interval 15
```

## Live Keys

- `q` or `Esc`: quit
- `Tab` or arrow keys: switch provider
- `s`: toggle Claude main or subagent usage
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
- Claude Code usage is deduplicated by `requestId`, keeping the latest usage record for each request.
- Claude subagent usage is tracked separately from the main Claude Code totals.

## Data Sources

- Claude Code: `~/.claude/projects/**/*.jsonl`
- Codex: `~/.codex/sessions/<year>/<month>/<day>/*.jsonl`
