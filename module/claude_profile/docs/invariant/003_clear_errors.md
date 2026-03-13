# Invariant: Clear Error Messages

### Scope

- **Purpose**: Ensure users always receive actionable guidance when something goes wrong.
- **Responsibility**: Documents the actionable error message requirement for all `claude_profile` errors (NFR-4).
- **In Scope**: Error messages from all CLI commands and library functions.
- **Out of Scope**: Log-level diagnostic output (separate concern), panic messages (panic = bug, not expected error).

### Invariant Statement

All errors produced by `claude_profile` must provide actionable messages — messages that tell the user what went wrong and what they can do next.

**Measurable threshold:** Every `CliError` variant that can reach the user has an error message that names the relevant file path, account name, or parameter — never a generic "operation failed".

**Required content for actionable errors:**
- Name the resource that caused the error (file path, account name, parameter name)
- State what was expected vs. what was found (when applicable)
- Suggest the corrective action (when obvious)

**Examples:**

```
✅ Good: "account 'work' not found in ~/.claude/accounts/"
✅ Good: "cannot delete active account 'work' — switch to another account first"
✅ Good: "~/.claude/.credentials.json not found — run 'claude' to authenticate first"

❌ Bad: "not found"
❌ Bad: "operation failed"
❌ Bad: "error reading file"
```

### Enforcement Mechanism

- Code review: every new `CliError` variant must pass the actionability test above
- Convention: errors include the account name or file path in the message string
- No generic error wrappers that swallow context

### Violation Consequences

- Users see cryptic errors and don't know how to recover
- Support burden increases — users need external help to diagnose common issues
- Reduces trust in the tool, especially for operations on credential files where mistakes matter

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/commands.rs` | Error message formatting for all CLI command handlers |
| source | `src/account.rs` | Account CRUD error messages (NotFound, PermissionDenied) |
