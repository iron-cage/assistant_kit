# json_redact

Domain-agnostic redaction of sensitive values from strings and JSON.

### Scope

Scrubs values whose key name contains a configurable deny-list atom (case-insensitive substring matching, so `accessToken`/`api_key`/`client_secret` are covered without enumeration) out of JSON documents and out of `key=value`/`key::value` pairs in free text such as CLI invocation strings. Independently of key names, secret-shaped values (`sk-ant-…` tokens, `eyJ…` JWTs, the token after a `Bearer` marker) are scrubbed wherever they appear. Deliberately biased toward over-redaction; the recursion depth guard fails closed. Pure library, no I/O, zero dependency on any `claude_*` crate.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `src/lib.rs` | RedactionPolicy, redact_json, redact_str |
| `verb/` | Shell scripts implementing do-protocol verbs for this crate. |
| `docs/` | Public API contract |
| `tests/` | Test Matrix coverage for redaction behavior |
