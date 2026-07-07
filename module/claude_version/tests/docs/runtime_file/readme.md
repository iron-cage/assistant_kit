# Runtime File Doc Entity

### Scope

- **Purpose**: Test case specifications for claude_version runtime file doc instances.
- **Responsibility**: Per-runtime-file RF- test specs covering path correctness, lifecycle, and durability.
- **In Scope**: Path accuracy, lifecycle trigger verification, durability classification. All 3 runtime file instances.
- **Out of Scope**: CLI discovery command tests (→ `../cli/command/15_runtime_files.md`, `../cli/command/16_paths.md`), network failure handling (→ `../feature/001_version_management.md`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `readme.md` | Index and overview of runtime file test specs |
| `001_version_history_cache.md` | RF- test cases for version history cache file |
| `002_versions_directory.md` | RF- test cases for versions directory |
| `003_binary_symlink.md` | RF- test cases for binary symlink |

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| 001_version_history_cache.md | RF- test cases for version_history_cache.json path, lifecycle, and durability | ⏳ |
| 002_versions_directory.md | RF- test cases for versions directory path, lifecycle, and durability | ⏳ |
| 003_binary_symlink.md | RF- test cases for binary symlink path, lifecycle, and durability | ⏳ |
