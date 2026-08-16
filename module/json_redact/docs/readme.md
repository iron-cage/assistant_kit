# docs/

### Scope

**Responsibilities:** API contract for the `json_redact` crate.
**In Scope:** Public redaction API (`RedactionPolicy`, `redact_str`, `redact_json`).
**Out of Scope:** Source code (-> `src/`), automated tests (-> `tests/`), caller-specific wiring (-> consuming crates' own docs).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `api/` | Public library API contract: RedactionPolicy, redact_str, redact_json |
