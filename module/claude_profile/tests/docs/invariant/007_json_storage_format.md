# Test: Invariant 007 — JSON Storage Format

Property assertion cases for `docs/invariant/007_json_storage_format.md`. Verifies that
all `.json` files written to disk by `clp` use `serde_json::to_string_pretty` and a
trailing newline — no minified JSON at any write site.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | No `serde_json::to_string(` calls in production source | Invariant holds (static) |
| IN-2 | Credential snapshot written by `.account.save` is pretty-printed with trailing newline | Invariant holds (runtime) |

**Total:** 2 IN cases

---

### IN-1: No `serde_json::to_string(` calls in production source

- **Given:** The `src/` directories of `module/claude_profile_core` and `module/claude_profile`
- **When:** `grep -r 'serde_json::to_string(' module/claude_profile_core/src/ module/claude_profile/src/` is executed
- **Then:** Command produces no matches — zero occurrences of `serde_json::to_string(` in
  production Rust source; all write sites use `serde_json::to_string_pretty`
- **Source:** [docs/invariant/007_json_storage_format.md](../../../docs/invariant/007_json_storage_format.md)

---

### IN-2: Credential snapshot written by `.account.save` is pretty-printed with trailing newline

- **Given:** A credential store with no existing account for `test@example.com`
- **When:** `clp .account.save name::test@example.com token::test-token` is executed
  and the resulting `test@example.com.json` file is read from disk
- **Then:** The file content is multi-line (not a single line), uses 2-space indentation,
  and ends with a single newline character (`\n`) — consistent with `serde_json::to_string_pretty`
  output plus an appended `\n`
- **Source:** [docs/invariant/007_json_storage_format.md](../../../docs/invariant/007_json_storage_format.md)
