# Event Schema

**Status**: Implemented | **Since**: 1.3.0

### Scope

- **Purpose**: Define the type-discriminated, version-tagged schema every journal event record follows.
- **Responsibility**: Documents the 9 event types, their trigger conditions and key fields, and the forward-compatible versioning rules.
- **In Scope**: The `v`/`type` discriminator fields, timestamp format, and per-event-type field population.
- **Out of Scope**: Journal write/read mechanics (→ `docs/feature/001_event_journaling.md`), the Rust type definitions (→ `docs/api/003_event_type.md`).

## Description

Type-discriminated, version-tagged event schema for the journal system. Every event record contains a `v` field (schema version, currently `1`) and a `type` field (event type discriminator). The schema uses a flat field bag where each event type populates a relevant subset of fields and leaves the rest omitted.

Nine event types are defined in schema v1:

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
| `command` | Every CLI invocation of any workspace binary | user, host, args, exit_code, duration_ms |

Timestamps use ISO 8601 format with millisecond precision in UTC (`2026-06-27T14:30:00.123Z`).

**Attribution fields** — `user`, `host`, `account`, and `agent_id` may appear on ANY event type (emitters populate them uniformly, not per-type):

| Field | Content |
|-------|---------|
| `user` | Username the event was produced by |
| `host` | Hostname the event was produced on |
| `account` | Non-secret account identifier the event was produced under (email or profile name) — never a token or secret |
| `agent_id` | Canonical agent identity `{user}@{host}{abs_dir}/` (e.g. `user1@w003/home/user1/pro/lib/yrd_core/assistant_kit/claude_runner/module/claude_runner/`), composed via `compose_agent_id()` — the single format owner |

The `v` field enables forward-compatible parsing: readers skip unknown fields; schema v2 events can add structure without breaking v1 readers. Unknown event types are preserved as raw JSON on read.

## Acceptance Criteria

- AC-001: Every event record contains `v`, `ts`, and `type` as non-null required fields
- AC-002: The `v` field is `1` for all events emitted by this version
- AC-003: The `type` field matches one of the 9 defined EventType variants exactly
- AC-004: The `ts` field is ISO 8601 with millisecond precision in UTC
- AC-005: Fields not relevant to an event type are **omitted** from JSON serialization (not serialized as null)
- AC-006: Unknown fields in a JSON line are silently ignored on deserialization (forward compat)
- AC-007: The `retry_class_counts` field is a 6-element array `[Transient, Account, Auth, Service, Process, Unknown]`
- AC-008: Numeric fields (`cost_usd`, `duration_ms`, `input_tokens`, `output_tokens`) use their native JSON types (number), not strings
- AC-009: `agent_id`, when present, matches `{user}@{host}{abs_dir}/` with exactly one trailing slash — composed via `compose_agent_id()`, never hand-concatenated
- AC-010: `account`, when present, is a non-secret identifier (email or profile name) — never a token, secret, or credential material
- AC-011: A pre-attribution JSONL line (no `account`/`agent_id` keys) still deserializes; both fields read as absent (additive, backward-compatible schema change)

## Sources

- `src/event.rs` — EventType enum, EventRecord struct, EventFields
- `claude_runner_core/src/types.rs` — ErrorKind enum (maps to error_class/error_kind fields)
