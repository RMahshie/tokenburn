# tokenburn -- Implementation Plan

## Context

Build a Rust CLI tool called `tokenburn` that displays token usage for Claude Code and OpenAI Codex in a TUI dashboard. The user wants a polished, fast tool installable via Homebrew. Two modes: one-shot (default, prints and exits) and live dashboard (`--live`).

## Data Sources

### Claude Code -- `~/.claude/projects/*/*.jsonl`
- Each line is JSON. Assistant messages have `message.usage` with: `input_tokens`, `output_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`
- Timestamp: top-level `timestamp` field (ISO 8601 with Z)
- Model: `message.model`
- Parse ALL session files, even for lifetime

### Codex -- `~/.codex/sessions/<year>/<month>/<day>/*.jsonl`
- Lines with `type: "event_msg"` and `payload.type: "token_count"` contain usage
- `payload.info.total_token_usage`: `input_tokens`, `cached_input_tokens`, `output_tokens`, `reasoning_output_tokens`
- Values are **cumulative** within a session -- use the LAST `token_count` entry per file
- Date extractable from directory path

## File Structure

```
tokenburn/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, dispatch one-shot vs live
│   ├── cli.rs               # clap arg definitions
│   ├── config.rs            # Paths, constants
│   ├── data/
│   │   ├── mod.rs
│   │   ├── types.rs         # TokenRecord, DailyBucket, ToolSummary, DashboardData
│   │   ├── claude.rs        # Claude JSONL parser
│   │   ├── codex.rs         # Codex JSONL parser
│   │   └── aggregator.rs    # Time filtering, bucketing, % change
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── app.rs           # App state (range, paused, quit, refresh timer)
│   │   ├── render.rs        # Top-level layout
│   │   ├── oneshot.rs       # One-shot: Viewport::Inline, render once, exit
│   │   ├── live.rs          # Live: alternate screen, event loop, auto-refresh
│   │   └── widgets/
│   │       ├── mod.rs
│   │       ├── token_table.rs   # Breakdown table with bars + sparklines
│   │       ├── line_chart.rs    # Token burn over time chart
│   │       ├── header.rs        # Header bar
│   │       ├── bottom_bar.rs    # Keybinding help
│   │       └── change_badge.rs  # % change arrow badge
│   └── util.rs              # format_tokens(), sparkline_string(), etc.
└── homebrew/
    └── tokenburn.rb          # Homebrew formula template
```

## Dependencies

- `clap` (derive) -- CLI args
- `serde` + `serde_json` -- JSON parsing
- `chrono` (serde feature) -- dates/times
- `ratatui` + `crossterm` -- TUI rendering
- `rayon` -- parallel file parsing
- `color-eyre` -- error handling
- `dirs` -- home directory
- `glob` -- file discovery

## Implementation Phases

### Phase 1: Data Layer
1. Define domain types in `data/types.rs`: `TokenRecord` (timestamp, tool enum, input/output/cache_create/cache_read/reasoning as `u64`), `DailyBucket`, `ToolSummary`, `DashboardData`
2. `data/claude.rs`: glob session files, rayon parallel parse, extract usage from assistant messages, return `Vec<TokenRecord>`
3. `data/codex.rs`: glob session files by directory structure, rayon parallel parse, take last `token_count` per file, return `Vec<TokenRecord>`
4. `data/aggregator.rs`: filter by time range, bucket by day, compute totals/daily avg/% of total, compute prior-period % change

### Phase 2: CLI + One-Shot Mode
1. `cli.rs`: clap derive with `--live`, `--range` (24h/7d/30d/lifetime), `--from`/`--to` for custom, `--interval`
2. `main.rs`: parse args, load data, dispatch to oneshot or live
3. `ui/oneshot.rs`: use `Viewport::Inline` to render into scrollback (not alternate screen)
4. Build out `render.rs` and all widgets: header, token_table (with inline unicode sparklines and block-char bars), line_chart (ratatui `Chart` widget), bottom_bar, change_badge

### Phase 3: Live Dashboard
1. `ui/app.rs`: state struct with range, paused, quit, refresh timer, help visibility
2. `ui/live.rs`: enter alternate screen, poll-based event loop (250ms tick), keybindings: q=quit, r=cycle range, p=pause, ?=help
3. Auto-refresh: re-parse all files on interval (default 5s)

### Phase 4: Polish
1. Graceful handling when only one tool is installed (show just that section)
2. Empty state for no data in range
3. Color scheme matching mockup (dark bg, green accents, blue bars, white text)
4. Sparklines rendered as unicode block chars in table cells

### Phase 5: Distribution
1. Homebrew formula in `homebrew/tokenburn.rb`
2. Tap repo setup instructions

## UI Layout

```
┌─────────── Header (name, version, range, refresh status) ──────────┐
├────────────────────────── CLAUDE CODE ─────────────── ▲ 18.7% ─────┤
│ METRIC    │ TOTAL      │ DAILY AVG  │ % TOTAL │ BAR  │ TREND (7D) │
│ Input     │ 1,234M     │ 176K       │ 45.6%   │ ████ │ ▂▃▅█▆▃▅   │
│ Output    │ 567M       │ 81K        │ 21.0%   │ ██   │ ▃▄▅▆▅▄▃   │
│ Cache Rd  │ 823M       │ 117M       │ 30.4%   │ ███  │ ▅▆▇█▇▆▅   │
│ Cache Wr  │ 82M        │ 11M        │ 3.0%    │ ▌    │ ▂▂▃▂▃▂▂   │
│ TOTAL     │ 2,708M     │ 387M       │ 100%    │ ████████████████  │
├──── Token burn over time (7D) ─── ratatui Chart widget ───────────┤
├──────────────────────────── CODEX ─────────────── ▲ 7.3% ─────────┤
│ (same layout + Reasoning row, Cached Input instead of Cache Wr)   │
├──── Token burn over time (7D) ────────────────────────────────────┤
├─── q Quit   r Range   p Pause   ? Help ──── Data as of: ... ─────┤
└───────────────────────────────────────────────────────────────────┘
```

## Key Decisions
- Sparklines as unicode strings in table cells (ratatui tables don't support embedded widgets)
- One-shot uses `Viewport::Inline` so output stays in scrollback
- Re-parse all files on refresh -- fast enough with rayon, avoids file-watcher complexity
- Codex "Reasoning" shown as own row, Claude doesn't expose it
- Codex "Cached Input" maps to Claude's "Cache Read" conceptually
- All timestamps normalized to UTC internally, displayed in local time

## Verification
1. `cargo build` compiles cleanly
2. `cargo run` (one-shot) displays both Claude and Codex tables with real data
3. `cargo run -- --live` opens interactive dashboard, keybindings work
4. `cargo run -- --range 24h` / `7d` / `30d` / `lifetime` all show correct filtered data
5. `cargo run -- --from 2026-04-01 --to 2026-04-30` custom range works
6. % change badge shows correct comparison vs prior period
7. Sparklines and line charts reflect daily token patterns
8. Works when only one tool's data exists (no crash)
9. Works with empty data in range (shows zero/empty state)
