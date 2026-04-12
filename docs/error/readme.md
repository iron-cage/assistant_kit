# Error Doc Entity

### Scope

- **Purpose**: Catalog error messages returned by the Claude Code binary during operation.
- **Responsibility**: Document each error's trigger conditions, plain-language meaning, and recovery steps.
- **In Scope**: Terminal error messages emitted by the `claude` binary; API, network, and authentication failures.
- **Out of Scope**: Session behaviors (→ `claude_code/001_session_behaviors.md`); internal Rust crate error types (→ source `src/`).
- **Type-Specific Sections**: Each doc instance MUST include, in addition to Common Doc Instance Requirements: Abstract (H3), Trigger Conditions (H3), Recovery (H3).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Rate Limit Reached](001_rate_limit_reached.md) | Document the API rate-limit error (HTTP 429) and recovery | ✅ |
| 002 | [Authentication Failed](002_authentication_failed.md) | Document auth errors: HTTP 401, OAuth failures, expired tokens | ✅ |
| 003 | [Context Limit Reached](003_context_limit_reached.md) | Document context-window overflow errors and compaction recovery | ✅ |
| 004 | [Request Timed Out](004_request_timed_out.md) | Document timeout error with 10-attempt exponential-backoff retry | ✅ |
| 005 | [API Overloaded](005_api_overloaded.md) | Document HTTP 529 capacity overload and retry guidance | ✅ |
