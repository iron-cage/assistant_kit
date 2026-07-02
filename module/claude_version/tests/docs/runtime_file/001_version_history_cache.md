# Runtime File Test: Version History Cache

### Scope

- **Purpose**: RF- test cases for the version_history_cache.json runtime file — path correctness, lifecycle triggers, and durability.
- **Responsibility**: Verify the cache file path spec, creation behavior, and safe-to-lose durability classification.
- **In Scope**: Path format, HOME expansion, creation via `.version.history`, durability after deletion.
- **Out of Scope**: Cache content and JSON schema (→ `../feature/001_version_management.md`), discovery command output (→ `../cli/command/15_runtime_files.md`).

Runtime file test surface for version_history_cache. See [runtime_file/001_version_history_cache.md](../../../docs/runtime_file/001_version_history_cache.md) for specification.

## Behavioral Divergence Pair

Two `.version.history` invocations that produce different file system outcomes:

- **Input A:** First call with no cache present → cache file created at `$HOME/.claude/.transient/version_history_cache.json`
- **Input B:** Subsequent call within TTL (< 3600 s) → cache file read, no write, mtime unchanged

## Test Case Index

| RF | Scenario | Source fn |
|----|----------|-----------|
| RF-1 | Path matches `$HOME/.claude/.transient/version_history_cache.json` exactly | ⏳ |
| RF-2 | `.version.history` creates cache file on first invocation when absent | ⏳ |
| RF-3 | `.version.history` succeeds after cache file is manually deleted (durability classification) | ⏳ |

## Test Coverage Summary

- Path correctness: 1 test (RF-1)
- Lifecycle creation: 1 test (RF-2)
- Durability: 1 test (RF-3)

**Total:** 3 tests

---

### RF-1: path matches spec — $HOME expansion

- **Given:** `HOME=/tmp/rf_test_home`
- **When:** `.runtime_files` output is examined
- **Then:** stdout contains exactly `/tmp/rf_test_home/.claude/.transient/version_history_cache.json`; path begins with HOME value; path ends with `.claude/.transient/version_history_cache.json`
- **Exit:** 0
- **Source:** [runtime_file/001_version_history_cache.md — Path](../../../docs/runtime_file/001_version_history_cache.md)

---

### RF-2: file created by first .version.history call when absent

- **Given:** `HOME=/tmp/rf_test_home` where `version_history_cache.json` does NOT exist; network accessible
- **When:** `clv .version.history`
- **Then:** exit 0; `$HOME/.claude/.transient/version_history_cache.json` exists on disk after the call; file contains a JSON array
- **Exit:** 0
- **Source:** [runtime_file/001_version_history_cache.md — Lifecycle: Created](../../../docs/runtime_file/001_version_history_cache.md)

---

### RF-3: durability — deletion is safe, next call re-creates

- **Given:** `HOME=/tmp/rf_test_home`; cache file exists at expected path; cache file is then manually deleted
- **When:** `clv .version.history` is called after deletion
- **Then:** exit 0; command succeeds despite missing cache; cache file is re-created at expected path after the call
- **Exit:** 0
- **Source:** [runtime_file/001_version_history_cache.md — Durability](../../../docs/runtime_file/001_version_history_cache.md)

---

## Source Functions

| Function | File | Test Cases |
|----------|------|------------|
| *(not yet implemented)* | `tests/cli/runtime_files_test.rs` | RF-1 through RF-3 |
