# Runtime File Test: Versions Directory

### Scope

- **Purpose**: RF- test cases for the versions directory runtime file — path correctness, lifecycle triggers (create/purge/lock), and durability.
- **Responsibility**: Verify the versions directory path spec, subdirectory creation/purge behavior, permission toggling, and recoverable durability classification.
- **In Scope**: Path format, HOME expansion, creation via `.version.install`, purge via pinned install, chmod lock toggle, durability after deletion.
- **Out of Scope**: Symlink retarget (→ `003_binary_symlink.md`), discovery command output (→ `../cli/command/16_paths.md`), chmod side effects on other processes (→ `../pitfall/001_version_lock_chmod.md`).

Runtime file test surface for the versions directory. See [runtime_file/002_versions_directory.md](../../../docs/runtime_file/002_versions_directory.md) for specification.

## Behavioral Divergence Pair

Two `.version.install` invocations that produce different file system outcomes:

- **Input A:** Install without pinning → target subdirectory added; other subdirectories left untouched
- **Input B:** Install with pin → target subdirectory added, then `purge_stale_versions()` removes all other subdirectories and locks the directory (555)

## Test Case Index

| RF | Scenario | Source fn |
|----|----------|-----------|
| RF-1 | Path matches `$HOME/.local/share/claude/versions` exactly | ⏳ |
| RF-2 | `.version.install` creates the directory and target subdirectory when absent | ⏳ |
| RF-3 | Pinned install purges all subdirectories except the kept version | ⏳ |
| RF-4 | `.version.install` succeeds after the directory is manually deleted (durability classification) | ⏳ |

## Test Coverage Summary

- Path correctness: 1 test (RF-1)
- Lifecycle creation: 1 test (RF-2)
- Purge behavior: 1 test (RF-3)
- Durability: 1 test (RF-4)

**Total:** 4 tests

---

### RF-1: path matches spec — $HOME expansion

- **Given:** `HOME=/tmp/rf_test_home`
- **When:** `.paths key::versions_dir` output is examined
- **Then:** stdout contains exactly `/tmp/rf_test_home/.local/share/claude/versions`; path begins with HOME value; path ends with `.local/share/claude/versions`
- **Exit:** 0
- **Source:** [runtime_file/002_versions_directory.md — Path](../../../docs/runtime_file/002_versions_directory.md)

---

### RF-2: directory and target subdirectory created by first install

- **Given:** `HOME=/tmp/rf_test_home` where the versions directory does NOT exist; network accessible
- **When:** `clv .version.install version::stable`
- **Then:** exit 0; `$HOME/.local/share/claude/versions/<resolved-version>/` exists on disk after the call
- **Exit:** 0
- **Source:** [runtime_file/002_versions_directory.md — Lifecycle: Created, Subdirectory added](../../../docs/runtime_file/002_versions_directory.md)

---

### RF-3: pinned install purges all other subdirectories

- **Given:** `HOME=/tmp/rf_test_home`; versions directory contains subdirectories for two prior versions
- **When:** `clv .version.install version::stable force::1` (pinned install)
- **Then:** exit 0; only the newly installed version's subdirectory remains under the versions directory; the two prior subdirectories are removed
- **Exit:** 0
- **Source:** [runtime_file/002_versions_directory.md — Lifecycle: Subdirectory removed](../../../docs/runtime_file/002_versions_directory.md)

---

### RF-4: durability — deletion is recoverable, next install re-creates

- **Given:** `HOME=/tmp/rf_test_home`; versions directory exists with content; directory is then manually deleted
- **When:** `clv .version.install version::stable` is called after deletion
- **Then:** exit 0; command succeeds despite the missing directory; directory is re-created with the target version's subdirectory after the call
- **Exit:** 0
- **Source:** [runtime_file/002_versions_directory.md — Durability](../../../docs/runtime_file/002_versions_directory.md)

---

## Source Functions

| Function | File | Test Cases |
|----------|------|------------|
| *(not yet implemented)* | `tests/cli/versions_dir_test.rs` | RF-1 through RF-4 |
