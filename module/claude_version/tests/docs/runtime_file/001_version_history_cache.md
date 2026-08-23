# Runtime File Test: Version History Cache

### Scope

- **Purpose**: RF- test cases for the version_history_cache.json runtime file — path correctness, lifecycle triggers, and durability.
- **Responsibility**: Verify the cache file path spec, creation behavior, and safe-to-lose durability classification.
- **In Scope**: Path format, HOME expansion, creation via `.version.list mode::history`, durability after deletion.
- **Out of Scope**: Cache content and JSON schema (→ `../feature/001_version_management.md`), discovery command output (→ `../cli/command/15_runtime_files.md`).

Runtime file test surface for version_history_cache. See [runtime_file/001_version_history_cache.md](../../../docs/runtime_file/001_version_history_cache.md) for specification.

## Behavioral Divergence Pair

Two `.version.list mode::history` invocations that produce different file system outcomes:

- **Input A:** First call with no cache present → cache file created at `$HOME/.claude/.transient/version_history_cache.json`
- **Input B:** Subsequent call within TTL (< 3600 s) → cache file read, no write, mtime unchanged

## Test Case Index

| RF | Scenario | Source fn |
|----|----------|-----------|
| RF-1 | Path matches `$HOME/.claude/.transient/version_history_cache.json` exactly | ✅ `it1_runtime_files_exits_0_with_cache_path`, `it5_custom_home_prefix`, `ft1_show_all_exits_0_with_cache_path`, `ft4_path_absolute_and_uses_home_expansion` |
| RF-2 | `.version.list mode::history` creates cache file on first invocation when absent | ⏳ blocked — requires network |
| RF-3 | `.version.list mode::history` succeeds after cache file is manually deleted (durability classification) | ⏳ blocked — requires network |

## Test Coverage Summary

- Path correctness: 1 case (RF-1) — ✅ implemented
- Lifecycle creation: 1 case (RF-2) — ⏳ blocked (network)
- Durability: 1 case (RF-3) — ⏳ blocked (network)

**Total:** 3 cases — 1 ✅ implemented, 2 ⏳ blocked

---

### RF-1: path matches spec — $HOME expansion

- **Given:** `HOME=/tmp/rf_test_home`
- **When:** `.runtime_files` output is examined
- **Then:** stdout contains exactly `/tmp/rf_test_home/.claude/.transient/version_history_cache.json`; path begins with HOME value; path ends with `.claude/.transient/version_history_cache.json`
- **Exit:** 0
- **Source:** [runtime_file/001_version_history_cache.md — Path](../../../docs/runtime_file/001_version_history_cache.md)

---

### RF-2: file created by first .version.list mode::history call when absent

- **Given:** `HOME=/tmp/rf_test_home` where `version_history_cache.json` does NOT exist; network accessible
- **When:** `clv .version.list mode::history`
- **Then:** exit 0; `$HOME/.claude/.transient/version_history_cache.json` exists on disk after the call; file contains a JSON array
- **Exit:** 0
- **Source:** [runtime_file/001_version_history_cache.md — Lifecycle: Created](../../../docs/runtime_file/001_version_history_cache.md)
- **Blocked (⏳):** requires network — `fetch_releases_json()` (`src/commands/history.rs`) writes the cache only after a successful `curl` fetch of the GitHub Releases API; offline the command falls back to the compiled-in `VERSION_HISTORY` snapshot and writes no file, so the "cache file exists after the call" assertion cannot hold.

---

### RF-3: durability — deletion is safe, next call re-creates

- **Given:** `HOME=/tmp/rf_test_home`; cache file exists at expected path; cache file is then manually deleted
- **When:** `clv .version.list mode::history` is called after deletion
- **Then:** exit 0; command succeeds despite missing cache; cache file is re-created at expected path after the call
- **Exit:** 0
- **Source:** [runtime_file/001_version_history_cache.md — Durability](../../../docs/runtime_file/001_version_history_cache.md)
- **Blocked (⏳):** requires network — the "command succeeds despite missing cache" half is already implied by `list_mode_tc2_history_shows_entries`, but the "cache file is re-created after the call" half needs the same successful GitHub Releases API fetch as RF-2.

---

## Source Functions

| Function | File | Test Cases |
|----------|------|------------|
| `it1_runtime_files_exits_0_with_cache_path` | `tests/cli/runtime_files_test.rs` | RF-1 |
| `it5_custom_home_prefix` | `tests/cli/runtime_files_test.rs` | RF-1 |
| `ft1_show_all_exits_0_with_cache_path` | `tests/cli/runtime_files_test.rs` | RF-1 |
| `ft4_path_absolute_and_uses_home_expansion` | `tests/cli/runtime_files_test.rs` | RF-1 |
| *(blocked — requires network)* | — | RF-2, RF-3 |
