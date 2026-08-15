# API Doc Entity

### Scope

**Responsibilities:** Public API contracts for the `json_redact` crate.
**In Scope:** RedactionPolicy construction, redact_str pattern-based redaction, redact_json key-name-based redaction.
**Out of Scope:** Internal recursion/depth-guard helpers, caller-specific policy extensions.

### Responsibility Table

| # | File | Responsibility |
|---|------|----------------|
| 001 | `001_redaction_api.md` | RedactionPolicy, redact_str, redact_json contract |
