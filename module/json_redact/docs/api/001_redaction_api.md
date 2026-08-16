# Redaction API

**Status**: Implemented | **Since**: 0.1.0

### Scope

- **Purpose**: Provide a domain-agnostic way to scrub sensitive values (tokens, credentials, secrets) from arbitrary strings and JSON content.
- **Responsibility**: Documents `RedactionPolicy`, `redact_str()`, and `redact_json()` — the crate's entire public surface — and their behavioral contract.
- **In Scope**: Key-name-based redaction over `serde_json::Value`, value-pattern scrubbing of secret-shaped content (`sk-ant-…`, JWTs, `Bearer` tokens), pattern-based redaction over free text, default and caller-extended deny-lists.
- **Out of Scope**: Redacting already-persisted files (in-memory transform only — no I/O), any `claude_*`-specific or journal-specific key names in the built-in deny-list.

## Description

`RedactionPolicy` holds a case-insensitive set of key-name atoms treated as sensitive; a key matches when its lowercased form *contains* any atom (substring matching, so `accessToken`, `refresh_token`, and `sessionKey` are all covered without enumerating variants). `RedactionPolicy::default()` seeds it with a domain-agnostic deny-list of 9 atoms. `redact_json` walks a `serde_json::Value` tree recursively (bounded to 64 levels of depth as a stack-overflow guard that *fails closed* — subtrees at the bound are replaced with `***REDACTED***`, never passed through) and replaces the value of any object key matching the policy; every other string, keys included, is additionally scrubbed for secret-shaped substrings. `redact_str` inspects whitespace-delimited tokens (whitespace runs are preserved verbatim) and, for each `key=value`/`key::value`-shaped token (with an optional leading `--` flag prefix), replaces the value with `***REDACTED***` when the key matches the policy — a redacted value that opens an unclosed quote swallows its quoted continuation across following tokens. Independently of key names, `sk-ant-…` token runs, `eyJ…` JWT shapes, and the token following a standalone `Bearer` word are scrubbed wherever they appear. The crate deliberately errs toward over-redaction (`monkey=1` matches the `key` atom): scrubbing a benign value is acceptable; leaking a credential is not.

## Interface

```rust
pub const REDACTED : &str = "***REDACTED***";

pub struct RedactionPolicy { /* private */ }

impl RedactionPolicy
{
  /// Returns a new policy with an additional sensitive key-name atom (case-insensitive,
  /// matched as a substring of the key).
  pub fn with_key( mut self, key : impl Into< String > ) -> Self;
}

impl Default for RedactionPolicy
{
  /// Built-in deny-list atoms: token, password, passwd, pwd, secret, auth, bearer, key, credential.
  fn default() -> Self;
}

/// Recursively redacts sensitive content in `value`, at any nesting depth.
pub fn redact_json( value : &serde_json::Value, policy : &RedactionPolicy ) -> serde_json::Value;

/// Redacts sensitive content in free text such as CLI invocation strings.
pub fn redact_str( input : &str, policy : &RedactionPolicy ) -> String;
```

## Behavioral Contract

- Key matching is case-insensitive substring matching against deny-list atoms (`API_KEY` and `accessToken` both match; over-redaction of benign atom-containing keys like `monkey` is deliberate)
- Value-pattern scrubbing runs regardless of key names: `sk-ant-…` token runs, `eyJ…` JWT shapes (≥ 2 dots, ≥ 20 chars), and the ≥ 8-char token following a standalone `Bearer` word are replaced wherever they appear, including mid-string and in JSON keys
- `redact_json` recurses into nested objects and arrays at any depth, up to an internal 64-level guard that fails closed — subtrees at the bound are replaced with `***REDACTED***` rather than returned unredacted
- Non-sensitive keys and non-pattern-matching values pass through structurally unchanged
- `redact_str` recognizes both `=` and `::` as key/value separators, preserves a leading `--` flag prefix and all whitespace runs (spaces, tabs, newlines) verbatim, and swallows the quoted continuation of a redacted value whose quote does not close within its own token
- Both entry points are pure functions — no I/O, no global/static mutable state, no panics on empty or multi-byte input
- The crate has zero dependency on any `claude_*` crate — a leaf, reusable by any caller

## Sources

- `src/lib.rs` — implementation
- `tests/redaction_test.rs` — Test Matrix T01-T20 coverage
- `task/json_redact/completed/468_create_json_redact_crate.md` — originating task, full Test Matrix and Acceptance Criteria
