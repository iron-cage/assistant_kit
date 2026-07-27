# Test: Invariant — Tools Array Doc Sync

Test case planning for [invariant/015_tools_array_doc_sync.md](../../../docs/invariant/015_tools_array_doc_sync.md). Tests validate that `src/cli/tools.rs`'s hardcoded `TOOLS` array stays bijective with `contract/claude_code/docs/tool/readme.md`'s Tool Table — every tool name+category pair present on one side must be present on the other, and the two collections must be the same size.

**Source:** [invariant/015_tools_array_doc_sync.md](../../../docs/invariant/015_tools_array_doc_sync.md)
**Related:** [command/08_tools.md](../../../docs/cli/command/08_tools.md) (`tools` command spec consuming `TOOLS`)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TS-1 | `TOOLS.len()` equals the contract doc's Tool Table row count | Invariant Hold |
| TS-2 | Every `TOOLS` entry's name+category has a matching row in the contract doc (forward bijection) | Invariant Hold |
| TS-3 | Every contract doc Tool Table row's name has a matching `TOOLS` entry (reverse bijection — the failure mode this invariant closes) | Invariant Hold |
| TS-4 | The sync guard reads the checked-in contract doc file directly — no network fetch, deterministic under test isolation | Implementation Constraint |

## Test Coverage Summary

- Invariant Hold: 3 tests (TS-1, TS-2, TS-3)
- Implementation Constraint: 1 test (TS-4)

**Total:** 4 invariant test cases

## Architectural Constraint

All 4 cases are unit tests co-located with the crate in `tests/tools_command_test.rs` — this file already exists, containing the 9 command-integration tests `it1_tools_exits_zero`–`it9_tools_rejects_unknown_arg` (see [command/08_tools.md](../../../docs/cli/command/08_tools.md)'s IT-1–IT-9); the 4 sync-guard cases here are new functions appended to that same file, not a new file. Each parses `contract/claude_code/docs/tool/readme.md`'s Tool Table into an in-memory `(name, category)` list at test time via a plain-text table-row scan (no markdown-parsing dependency needed — the table's row format is stable and simple enough for a line-oriented scan), then compares against `tools::TOOLS`. Implementation tracked under `task/claude_runner/`.

## Implementation Notes

| ID | Test Function | File | Status |
|----|---------------|------|--------|
| TS-1 | `tools_array_count_matches_contract_doc` | `tests/tools_command_test.rs` | ⏳ |
| TS-2 | `tools_array_forward_bijection_with_contract_doc` | `tests/tools_command_test.rs` | ⏳ |
| TS-3 | `tools_array_reverse_bijection_with_contract_doc` | `tests/tools_command_test.rs` | ⏳ |
| TS-4 | `tools_array_sync_guard_reads_checked_in_doc` | `tests/tools_command_test.rs` | ⏳ |

**Note on implementation status:** all 4 cases are `⏳` — this invariant and its enforcement test were newly documented during the 2026-07 `clr tools` filter/projection/inspect redesign, alongside the discovery that `TOOLS` held only 26 of the contract doc's 40 entries. Implementation is tracked under `task/claude_runner/`.

---

### TS-1: `TOOLS.len()` equals contract doc row count

- **Given:** `contract/claude_code/docs/tool/readme.md`'s Tool Table, parsed into a row list
- **When:** `tools::TOOLS.len()` is compared against the parsed row count
- **Then:** the two counts are equal
- **Note:** pins the exact failure mode that motivated this invariant — `TOOLS` held only 26 entries while the contract doc listed 40
- **Source:** [invariant/015_tools_array_doc_sync.md](../../../docs/invariant/015_tools_array_doc_sync.md) § Invariant Statement

---

### TS-2: Forward bijection — every `TOOLS` entry exists in the contract doc

- **Given:** the parsed contract doc row list and `tools::TOOLS`
- **When:** each `TOOLS` entry's `(name, category)` pair is looked up in the parsed row list
- **Then:** every entry finds a match — no `TOOLS` entry names a tool absent from the contract doc, and no entry's category diverges from the contract doc's category for that tool
- **Source:** [invariant/015_tools_array_doc_sync.md](../../../docs/invariant/015_tools_array_doc_sync.md) § Invariant Statement

---

### TS-3: Reverse bijection — every contract doc row exists in `TOOLS`

- **Given:** the parsed contract doc row list and `tools::TOOLS`
- **When:** each parsed row's tool name is looked up in `tools::TOOLS`
- **Then:** every row finds a match — no contract doc tool is missing from `TOOLS`
- **Note:** this is the exact direction that would have caught the original defect — the forward direction alone (TS-2) is satisfied trivially by a *subset*; only the reverse direction detects an incomplete `TOOLS` array
- **Source:** [invariant/015_tools_array_doc_sync.md](../../../docs/invariant/015_tools_array_doc_sync.md) § Violation Consequences

---

### TS-4: Sync guard is hermetic — reads the checked-in doc file, not a live fetch

- **Given:** the crate's test environment, with no network access assumed
- **When:** the sync guard test runs
- **Then:** it reads `contract/claude_code/docs/tool/readme.md` from the checked-out working tree via a relative path — it does not fetch Claude Code's tool documentation from any external URL
- **Note:** keeps the guard deterministic and usable inside the container-only test execution environment ([invariant/010](../../../docs/invariant/010_container_only_test_execution.md))
- **Source:** [invariant/015_tools_array_doc_sync.md](../../../docs/invariant/015_tools_array_doc_sync.md) § Enforcement Mechanism
