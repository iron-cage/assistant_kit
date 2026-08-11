# Redaction API

**Status**: Implemented | **Since**: 0.1.0

### Scope

- **Purpose**: Provide a domain-agnostic way to scrub sensitive values (tokens, credentials, secrets) from arbitrary strings and JSON content.
- **Responsibility**: Documents `RedactionPolicy`, `redact_str()`, and `redact_json()` — the crate's entire public surface — and their behavioral contract.
- **In Scope**: Key-name-based redaction over `serde_json::Value`, pattern-based redaction over free text, default and caller-extended deny-lists.
- **Out of Scope**: Redacting already-persisted files (in-memory transform only — no I/O), any `claude_*`-specific or journal-specific key names in the built-in deny-list.

## Description

`RedactionPolicy` holds a case-insensitive set of key names treated as sensitive. `RedactionPolicy::default()` seeds it with a domain-agnostic deny-list of 8 common credential key shapes. `redact_json` walks a `serde_json::Value` tree recursively (bounded to 64 levels of depth as a stack-overflow guard) and replaces the value of any object key matching the policy with the literal string `***REDACTED***`, leaving non-matching keys and non-object/array values untouched. `redact_str` splits free text on spaces and, for each `key=value`/`key::value`-shaped token (with an optional leading `--` flag prefix), replaces the value with `***REDACTED***` when the key matches the policy; tokens with no separator (e.g. bare flags) pass through unchanged.

## Interface

```rust
pub const REDACTED : &str = "***REDACTED***";

pub struct RedactionPolicy { /* private */ }

impl RedactionPolicy
{
  /// Returns a new policy with an additional sensitive key name (case-insensitive).
  pub fn with_key( mut self, key : impl Into< String > ) -> Self;
}

impl Default for RedactionPolicy
{
  /// Built-in deny-list: token, password, secret, authorization, api_key, apikey, key, credential.
  fn default() -> Self;
}

/// Recursively redacts values in `value` whose key matches `policy`, at any nesting depth.
pub fn redact_json( value : &serde_json::Value, policy : &RedactionPolicy ) -> serde_json::Value;

/// Redacts `key=value`/`key::value` pairs in free text whose key matches `policy`.
pub fn redact_str( input : &str, policy : &RedactionPolicy ) -> String;
```

## Behavioral Contract

- Key matching is case-insensitive (`API_KEY` matches the `api_key` deny-list entry)
- `redact_json` recurses into nested objects and arrays at any depth, up to an internal 64-level guard beyond which content is returned unmodified rather than risking a stack overflow
- Non-sensitive keys and non-`Object`/`Array` values pass through structurally unchanged
- `redact_str` recognizes both `=` and `::` as key/value separators, and preserves a leading `--` flag prefix on the returned token
- Both entry points are pure functions — no I/O, no global/static mutable state, no panics on empty input
- The crate has zero dependency on any `claude_*` crate — a leaf, reusable by any caller

## Sources

- `src/lib.rs` — implementation
- `tests/redaction_test.rs` — Test Matrix T01-T09 coverage
- `task/json_redact/completed/468_create_json_redact_crate.md` — originating task, full Test Matrix and Acceptance Criteria
