# src/

Domain-agnostic redaction of sensitive values from strings and JSON content.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | `RedactionPolicy`, `redact_str()`, `redact_json()` |

### Scope

**In Scope:**
- Key-name-based redaction over `serde_json::Value`
- Value-pattern scrubbing of secret-shaped content (`sk-ant-…`, JWTs, `Bearer` tokens)
- Pattern-based redaction over free text; default and caller-extended deny-lists

**Out of Scope:**
- Redacting already-persisted files (in-memory transform only — no I/O)
- Any `claude_*`-specific or journal-specific key names in the built-in deny-list (this crate has zero dependency on any `claude_*` crate)

See [`docs/api/001_redaction_api.md`](../docs/api/001_redaction_api.md) for the full behavioral contract.
