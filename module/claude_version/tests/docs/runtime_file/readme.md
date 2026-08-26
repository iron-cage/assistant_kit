# Runtime File Doc Entity

### Scope

- **Purpose**: Test case specifications for claude_version runtime file doc instances.
- **Responsibility**: Per-runtime-file RF- test specs covering path correctness, lifecycle, and durability.
- **In Scope**: Path accuracy, lifecycle trigger verification, durability classification. All 4 runtime file instances.
- **Out of Scope**: CLI discovery command tests (→ `../cli/command/15_runtime_files.md`, `../cli/command/16_version_paths.md`), network failure handling (→ `../feature/001_version_management.md`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `readme.md` | Index and overview of runtime file test specs |
| [001_version_history_cache.md](001_version_history_cache.md) | RF- test cases for version history cache file |
| [002_versions_directory.md](002_versions_directory.md) | RF- test cases for versions directory |
| [003_binary_symlink.md](003_binary_symlink.md) | RF- test cases for binary symlink |
| [004_version_markers.md](004_version_markers.md) | RF- test cases for version markers file |

### Overview Table

| Name | Purpose | Cases | Implemented | Blocked | Status |
|------|---------|-------|-------------|---------|--------|
| 001_version_history_cache.md | RF- test cases for version_history_cache.json path, lifecycle, and durability | 3 | 1 | 2 | ⏳ |
| 002_versions_directory.md | RF- test cases for versions directory path, lifecycle, and durability | 4 | 1 | 3 | ⏳ |
| 003_binary_symlink.md | RF- test cases for binary symlink path, lifecycle, and durability | 3 | 1 | 2 | ⏳ |
| 004_version_markers.md | RF- test cases for version-markers.json path, lifecycle, and graceful degradation | 4 | 4 | 0 | ✅ |

**Total:** 14 RF cases — 7 ✅ implemented, 7 ⏳ blocked.

### Blocked Cases

All 7 ⏳ cases are blocked on the same two external dependencies, not on missing test effort:

| Blocker | Cases | Why |
|---------|-------|-----|
| Network — GitHub Releases API | 001 RF-2, RF-3 | `fetch_releases_json()` writes `version_history_cache.json` only after a successful `curl` fetch; offline the command falls back to the compiled-in snapshot and writes nothing |
| Network + real Claude Code installer | 002 RF-2, RF-3, RF-4; 003 RF-2, RF-3 | `perform_install()` shells out to `curl -fsSL <INSTALL_URL> \| bash`; only that installer creates version subdirectories and retargets the binary symlink. `dry::1` returns before it runs |

Path-correctness cases (RF-1 in every instance) are fully covered offline, since every
managed path is derived from `HOME` without touching disk.
