# Invariant: JSON Storage Format

### Scope

- **Purpose**: Ensure all `.json` files written by `clp` are human-readable and diff-friendly.
- **Responsibility**: Documents the 2-space pretty-print + trailing-newline requirement for every `serde_json` file write across `claude_profile_core` and its test infrastructure.
- **In Scope**: All `std::fs::write` / `std::fs::rename` calls that produce `.json` files — credential snapshots (`{name}.json`), `~/.claude.json`, `~/.claude/settings.json`.
- **Out of Scope**: In-memory serialization for HTTP request bodies; hand-rolled formatters (e.g., `json_serialize_flat_object` in `claude_core/src/settings_io.rs` already produces pretty output).

### Invariant Statement

Every `.json` file written to disk by `clp` must use `serde_json::to_string_pretty` and append a trailing newline (`\n`).

**Measurable threshold:** Zero occurrences of `serde_json::to_string(` in production Rust source files under `src/` (only `to_string_pretty` is permitted). Verified by:
```
grep -r 'serde_json::to_string(' module/claude_profile_core/src/ module/claude_profile/src/
```
Expected result: no matches.

**Guarantee:** All `.json` files on disk are multi-line, 2-space-indented, and end with a single newline — consistent with `git diff` line-diff expectations and manual readability.

**Known exceptions:** `json_serialize_flat_object` in `module/claude_core/src/settings_io.rs` uses a hand-rolled formatter (single-level object, `"{\n  key: val,\n  ...\n}"`). This already produces pretty output and is exempt from the `to_string_pretty` rule.

### Enforcement Mechanism

- **Grep gate:** `grep -r 'serde_json::to_string(' src/` in any affected crate must return no matches.
- **Code review:** Reject any PR that introduces `serde_json::to_string(` for file writes; require `serde_json::to_string_pretty` + `\n` suffix.
- **Affected call sites (38 total):**
  - `module/claude_profile_core/src/account.rs` — 17 write sites (L331, L435, L561, L601, L620, L743, L781, L822, L856, L892, L1419, L1449, L1479, L1602, L2044, L2098, L2356)
  - `module/claude_profile/tests/usage/fetch_tests.rs` — 2 test-fixture write sites
  - `module/claude_profile/tests/usage/fetch_tests_b.rs` — 10 test-fixture write sites
  - `module/claude_profile/tests/cli/cli_runner.rs` — 1 test-fixture write site
  - `module/claude_profile/tests/cli/usage_solo_test.rs` — 7 test-fixture write sites
  - `module/claude_profile/tests/cli/model_test.rs` — 1 test-fixture write site

### Violation Consequences

- Minified JSON in credential/settings files makes `git diff` output unreadable (entire file on one line).
- Manual inspection and debugging require external formatting tools.
- Automated diff tests against expected fixture content will fail if format is inconsistent.

### Sources

| File | Relationship |
|------|-------------|
| `module/claude_profile_core/src/account.rs` | 17 JSON write sites requiring `to_string_pretty` |
| `module/claude_core/src/settings_io.rs` | `json_serialize_flat_object` — hand-rolled pretty formatter; permitted exception |

### Features

| File | Relationship |
|------|-------------|
| [010_persistent_storage.md](../feature/010_persistent_storage.md) | Credential store path `{root}/.persistent/claude/credential/` |
| [007_file_topology.md](../feature/007_file_topology.md) | `ClaudePaths` type — describes `.credentials.json`, `settings.json`, `~/.claude.json` |

### Tests

| File | Relationship |
|------|-------------|
| `module/claude_profile/tests/usage/fetch_tests.rs` | 2 test-fixture JSON write sites |
| `module/claude_profile/tests/usage/fetch_tests_b.rs` | 10 test-fixture JSON write sites |
| `module/claude_profile/tests/cli/cli_runner.rs` | 1 test-fixture JSON write site |
| `module/claude_profile/tests/cli/usage_solo_test.rs` | 7 test-fixture JSON write sites |
| `module/claude_profile/tests/cli/model_test.rs` | 1 test-fixture JSON write site |
