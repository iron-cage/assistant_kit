# json_redact

Domain-agnostic redaction of sensitive values from strings and JSON.

### Scope

Scrubs values whose key name matches a configurable deny-list (case-insensitive) out of JSON documents, and scrubs `key=value`/`key::value` pairs out of free text such as CLI invocation strings. Pure library, no I/O, zero dependency on any `claude_*` crate.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `src/lib.rs` | RedactionPolicy, redact_json, redact_str |
| `verb/` | Shell scripts implementing do-protocol verbs for this crate. |
| `docs/` | Public API contract |
| `tests/` | Test Matrix coverage for redaction behavior |
