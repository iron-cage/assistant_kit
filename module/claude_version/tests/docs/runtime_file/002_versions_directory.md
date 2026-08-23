# Runtime File Test: Versions Directory

### Scope

- **Purpose**: RF- test cases for the versions directory runtime file — path correctness, lifecycle triggers (create/purge/lock), and durability.
- **Responsibility**: Verify the versions directory path spec, subdirectory creation/purge behavior, permission toggling, and recoverable durability classification.
- **In Scope**: Path format, HOME expansion, creation via `.version.install`, purge via pinned install, chmod lock toggle, durability after deletion.
- **Out of Scope**: Symlink retarget (→ `003_binary_symlink.md`), discovery command output (→ `../cli/command/16_version_paths.md`), chmod side effects on other processes (→ `../pitfall/001_version_lock_chmod.md`).

Runtime file test surface for the versions directory. See [runtime_file/002_versions_directory.md](../../../docs/runtime_file/002_versions_directory.md) for specification.

## Behavioral Divergence Pair

Two `.version.install` invocations that produce different file system outcomes:

- **Input A:** Install without pinning → target subdirectory added; other subdirectories left untouched
- **Input B:** Install with pin → target subdirectory added, then `purge_stale_versions()` removes all other subdirectories and locks the directory (555)

## Test Case Index

| RF | Scenario | Source fn |
|----|----------|-----------|
| RF-1 | Path matches `$HOME/.local/share/claude/versions` exactly | ✅ `path_key_tc2_versions_dir_resolves`, `it02_paths_single_versions_dir`, `ft3_single_key_returns_one_path` |
| RF-2 | `.version.install` creates the directory and target subdirectory when absent | ⏳ blocked — requires network + real installer |
| RF-3 | Pinned install purges all subdirectories except the kept version | ⏳ blocked — requires network + real installer |
| RF-4 | `.version.install` succeeds after the directory is manually deleted (durability classification) | ⏳ blocked — requires network + real installer |

## Test Coverage Summary

- Path correctness: 1 case (RF-1) — ✅ implemented
- Lifecycle creation: 1 case (RF-2) — ⏳ blocked (network + real installer)
- Purge behavior: 1 case (RF-3) — ⏳ blocked (network + real installer)
- Durability: 1 case (RF-4) — ⏳ blocked (network + real installer)

**Total:** 4 cases — 1 ✅ implemented, 3 ⏳ blocked

---

### RF-1: path matches spec — $HOME expansion

- **Given:** `HOME=/tmp/rf_test_home`
- **When:** `.version.paths key::versions_dir` output is examined
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
- **Blocked (⏳):** requires network and a real Claude Code install — `perform_install()` (`claude_version_core/src/version.rs`) shells out to `curl -fsSL <INSTALL_URL> | bash`, and only that installer creates the version subdirectory. Every offline `.version.install` test in the suite uses `dry::1`, which returns before `perform_install()` is reached.

---

### RF-3: pinned install purges all other subdirectories

- **Given:** `HOME=/tmp/rf_test_home`; versions directory contains subdirectories for two prior versions
- **When:** `clv .version.install version::stable force::1` (pinned install)
- **Then:** exit 0; only the newly installed version's subdirectory remains under the versions directory; the two prior subdirectories are removed
- **Exit:** 0
- **Source:** [runtime_file/002_versions_directory.md — Lifecycle: Subdirectory removed](../../../docs/runtime_file/002_versions_directory.md)
- **Blocked (⏳):** requires network and a real Claude Code install — `purge_stale_versions()` runs only after `perform_install()` reports a verified successful install, which needs the live `curl … | bash` installer.

---

### RF-4: durability — deletion is recoverable, next install re-creates

- **Given:** `HOME=/tmp/rf_test_home`; versions directory exists with content; directory is then manually deleted
- **When:** `clv .version.install version::stable` is called after deletion
- **Then:** exit 0; command succeeds despite the missing directory; directory is re-created with the target version's subdirectory after the call
- **Exit:** 0
- **Source:** [runtime_file/002_versions_directory.md — Durability](../../../docs/runtime_file/002_versions_directory.md)
- **Blocked (⏳):** requires network and a real Claude Code install — the "directory is re-created with the target version's subdirectory" clause is produced by the live installer invoked from `perform_install()`.

---

## Source Functions

| Function | File | Test Cases |
|----------|------|------------|
| `path_key_tc2_versions_dir_resolves` | `tests/cli/path_key_test.rs` | RF-1 |
| `it02_paths_single_versions_dir` | `tests/cli/paths_test.rs` | RF-1 |
| `ft3_single_key_returns_one_path` | `tests/cli/paths_test.rs` | RF-1 |
| *(blocked — requires network + real installer)* | — | RF-2, RF-3, RF-4 |
