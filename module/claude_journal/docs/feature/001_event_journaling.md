# Event Journaling

**Status**: Planned | **Since**: 1.3.0

## Description

Append-only structured event logging for CLR automation sessions. Every CLR execution (print-mode, interactive, credential subprocess) emits a structured event record to a daily JSONL file. The journal captures the full lifecycle of each invocation — command, parameters, model, timing, exit code, error classification, retry history, cost, token usage, and complete stdout/stderr output.

Default journal level is `full` — capturing complete stdout and stderr content (truncated at 1MB per field to prevent unbounded growth from large code-generation responses). The `meta` level omits stdout/stderr while preserving all other fields. The `off` level disables journaling entirely.

Journal files are stored in a configurable directory (default `~/.clr/journal/`) with one file per UTC day named `YYYY-MM-DD.jsonl`. Each line is a self-contained JSON object parseable independently — no multi-line records, no header/footer, no file-level framing.

## Acceptance Criteria

- AC-001: `JournalWriter::append()` creates the daily file if absent, appends one JSON line, and returns `Ok(())`
- AC-002: Each JSON line is a complete, self-contained event record parseable by `serde_json::from_str`
- AC-003: Journal directory is configurable via `CLR_JOURNAL_DIR` env var (default `~/.clr/journal/`)
- AC-004: Journal level is configurable via `CLR_JOURNAL` env var with values `full` (default), `meta`, `off`
- AC-005: At `full` level, stdout and stderr fields are populated with complete subprocess output
- AC-006: At `full` level, stdout and stderr fields exceeding 1MB are truncated with a trailing `\n[truncated at 1MB]` marker
- AC-007: At `meta` level, stdout and stderr fields are `null` in the JSON record
- AC-008: At `off` level, no journal write occurs — `JournalWriter::append()` is never called
- AC-009: Appending to a journal file that is simultaneously read by `JournalReader` does not corrupt either operation
- AC-010: Journal directory is created automatically on first write if it does not exist

## Sources

- `src/writer.rs` — JournalWriter implementation
- `src/event.rs` — EventRecord and EventFields structs
- `claude_runner/src/cli/execution.rs` — integration call sites
