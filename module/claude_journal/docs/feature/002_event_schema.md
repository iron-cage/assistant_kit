# Event Schema

**Status**: Planned | **Since**: 1.3.0

## Description

Type-discriminated, version-tagged event schema for the journal system. Every event record contains a `v` field (schema version, currently `1`) and a `type` field (event type discriminator). The schema uses a flat field bag where each event type populates a relevant subset of fields and leaves the rest omitted.

Eight event types are defined in schema v1:

| Type | Trigger | Key Fields |
|------|---------|------------|
| `execution` | Every `run`/`ask` print-mode completion | command, message, dir, model, effort, timeout_secs, exit_code, duration_ms, error_class, error_kind, retries, retry_class_counts, cost_usd, input_tokens, output_tokens, session_id, creds, output_style, output_format, stdout, stderr, runner_version |
| `credential` | Every `isolated`/`refresh` subprocess completion | command, creds, exit_code, duration_ms, model, effort, stdout, stderr |
| `gate_wait` | Concurrency gate activation | max_sessions, wait_ms, gate_attempts, gate_outcome |
| `retry` | Each error-class retry attempt | error_class, error_kind, attempt, limit, delay_secs, message, exit_code |
| `timeout` | Watchdog kills subprocess | command, timeout_secs, pid |
| `runner_retry` | Each spawn failure retry | attempt, error_message, delay_secs |
| `validation_retry` | Each expect-validation retry | pattern, got, attempt, strategy |
| `interactive` | Every interactive session end | command, dir, model, timeout_secs, exit_code, duration_ms |

Timestamps use ISO 8601 format with millisecond precision in UTC (`2026-06-27T14:30:00.123Z`).

The `v` field enables forward-compatible parsing: readers skip unknown fields; schema v2 events can add structure without breaking v1 readers. Unknown event types are preserved as raw JSON on read.

## Acceptance Criteria

- AC-001: Every event record contains `v`, `ts`, and `type` as non-null required fields
- AC-002: The `v` field is `1` for all events emitted by this version
- AC-003: The `type` field matches one of the 8 defined EventType variants exactly
- AC-004: The `ts` field is ISO 8601 with millisecond precision in UTC
- AC-005: Fields not relevant to an event type are **omitted** from JSON serialization (not serialized as null)
- AC-006: Unknown fields in a JSON line are silently ignored on deserialization (forward compat)
- AC-007: The `retry_class_counts` field is a 6-element array `[Transient, Account, Auth, Service, Process, Unknown]`
- AC-008: Numeric fields (`cost_usd`, `duration_ms`, `input_tokens`, `output_tokens`) use their native JSON types (number), not strings

## Sources

- `src/event.rs` — EventType enum, EventRecord struct, EventFields
- `claude_runner_core/src/types.rs` — ErrorKind enum (maps to error_class/error_kind fields)
