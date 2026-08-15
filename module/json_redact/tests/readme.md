# tests/

### Scope

**Responsibilities:** Automated integration tests for the `json_redact` crate — key-name-based JSON redaction and free-text `key=value`/`key::value` pattern scrubbing.
**In Scope:** All crate functionality exercised via the public library API (`redact_json`, string pattern redaction, deny-list policy).
**Out of Scope:** Manual testing, test planning documents.

### Domain Map

| Domain | File | Tests What |
|--------|------|------------|
| Redaction (T01–T09) | `redaction_test.rs` | Key-name redaction (top-level, nested, case-insensitive), non-sensitive keys left untouched, array element redaction, `key=value`/`key::value` free-text pattern redaction, custom policy extension, empty input, deeply nested JSON, default deny-list cardinality |
