# Filtering

**Status**: Planned | **Since**: 1.3.0

## Description

Query filter system shared by all viewing commands (`.list`, `.search`, `.stats`, `.export`) and the web API. Filters are AND-combined: all non-empty filter parameters must match for an event to be included. The filter system maps CLI `param::value` arguments to `JournalFilter` fields.

Eight filter dimensions:

| Filter | Param | Match Type | Example |
|--------|-------|-----------|---------|
| Time (after) | `since::` | Duration parse, events after `now - duration` | `since::1h`, `since::7d` |
| Time (before) | `until::` | Duration parse, events before `now - duration` | `until::1d` |
| Event type | `type::` | Exact match against EventType variants | `type::execution` |
| CLR command | `command::` | Exact match: run, ask, isolated, refresh | `command::ask` |
| Exit code | `exit::` | Exact integer match | `exit::2` |
| Model | `model::` | Substring match (case-sensitive) | `model::opus` |
| Directory | `dir::` | Substring match (case-sensitive) | `dir::/home/user/project` |
| Credentials | `creds::` | Exact string match | `creds::default` |

Additionally, `limit::N` caps the number of returned events (default 50 for `.list`, unlimited for `.export`).

Duration parsing accepts: `Ns` (seconds), `Nm` (minutes), `Nh` (hours), `Nd` (days), `Nw` (weeks). No spaces between number and unit.

## Acceptance Criteria

- AC-001: Empty filter (no params) matches all events
- AC-002: Multiple filters combine as AND: `type::execution command::ask` matches only ask-execution events
- AC-003: `since::1h` correctly parses as "events from the last 60 minutes"
- AC-004: `model::opus` matches events with model containing "opus" as substring
- AC-005: `exit::0` matches only successful events; `exit::2` matches only rate-limit events
- AC-006: Invalid duration format (e.g., `since::abc`) produces an error message and exits 1
- AC-007: Invalid event type (e.g., `type::bogus`) produces an error message and exits 1
- AC-008: `limit::10` caps output to 10 events regardless of total matches

## Sources

- `src/cli/mod.rs` — filter construction from CLI args
- `claude_journal/src/reader.rs` — JournalFilter application
