# Feature :: CLI Tool

Feature-level test cases for the `claude_storage` CLI tool. Tests validate the tool's
two invocation modes, its transparent handling of both storage layouts, and its support
for both project identifier schemes.

**Source:** [001_cli_tool.md](../../../../docs/feature/001_cli_tool.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| FT-1 | One-shot command executes and exits cleanly | Invocation Mode |
| FT-2 | Unknown command rejected with non-zero exit | Invocation Mode |
| FT-3 | Path-encoded project returned in project list | Project Scheme |
| FT-4 | UUID-named project returned in project list | Project Scheme |
| FT-5 | Flat-layout (B7) project sessions accessible | Storage Layout |
| FT-6 | Hierarchical-layout (B13) project sessions accessible | Storage Layout |

## Test Coverage Summary

- Invocation Mode: 2 tests (FT-1, FT-2)
- Project Scheme: 2 tests (FT-3, FT-4)
- Storage Layout: 2 tests (FT-5, FT-6)

**Total:** 6 feature cases

## Test Cases

---

### FT-1: One-shot command executes and exits cleanly

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with at least one project in storage
- **When:** `clg .status`
- **Then:** stdout contains storage status output (root path, project count); process exits cleanly
- **Exit:** 0

---

### FT-2: Unknown command rejected with non-zero exit

- **Given:** clean environment
- **When:** `clg .nonexistent_command`
- **Then:** stderr contains an error indicating the command is unknown or unrecognized; no stack trace or panic
- **Exit:** 1

---

### FT-3: Path-encoded project returned in project list

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a path-encoded project directory (e.g., `-home-alice-projects-myproject`)
- **When:** `clg .list`
- **Then:** stdout includes the path-encoded project; the project identifier is present in the output
- **Exit:** 0

---

### FT-4: UUID-named project returned in project list

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a UUID-named project directory (e.g., `8d795a1c-c81d-4010-8d29-b4e678272419`)
- **When:** `clg .list`
- **Then:** stdout includes the UUID-named project; both path-encoded and UUID projects appear in the same listing
- **Exit:** 0

---

### FT-5: Flat-layout (B7) project sessions accessible

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a flat-layout project (`.jsonl` files directly in the project directory, no `sessions/` or `conversations/` subdirectory)
- **When:** `clg .list sessions::1`
- **Then:** sessions from the flat-layout project appear in the output; no error about unrecognized layout
- **Exit:** 0

---

### FT-6: Hierarchical-layout (B13) project sessions accessible

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` with a hierarchical-layout project (`.jsonl` files inside a `sessions/` or `conversations/` subdirectory)
- **When:** `clg .list sessions::1`
- **Then:** sessions from the hierarchical-layout project appear in the output alongside flat-layout sessions; both layouts surfaced in a single command
- **Exit:** 0
