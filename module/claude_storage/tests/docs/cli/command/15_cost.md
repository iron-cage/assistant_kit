# Command :: `.cost`

Integration tests for the `.cost` command, implemented in `tests/cli_cmd_cost_test.rs`. Tests verify default current-conversation resolution (single row, no TOTAL), multi-conversation selection with the TOTAL row and request-order preservation, exact/prefix session ID resolution across projects (ambiguous and unknown IDs rejected, duplicates collapsed, cross-project duplicate IDs tie-broken to the richest copy), agent fold-in across BOTH agent layouts with the `agents::0` opt-out, argument validation raised before any storage access, a hand-computed golden pricing render (cache-TTL split, unknown-TTL fallback, multi-model summation, `<synthetic>` skip, compaction count, max context), and exit codes. Core aggregation arithmetic (dedup by `message.id`, TTL clamping, model bucket ordering) is covered line-by-line in `claude_storage_core/tests/cost_report_test.rs` — these tests exercise the CLI contract end-to-end through the binary.

**Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| INT-1 | Bare `.cost` reports the current conversation as one row without TOTAL | Default Resolution |
| INT-2 | Multiple `session_ids::` produce rows in request order plus a TOTAL row | Multi-Conversation |
| INT-3 | A unique session ID prefix resolves to its conversation | ID Resolution |
| INT-4 | An ambiguous session ID prefix is rejected naming every match | ID Resolution |
| INT-5 | A session ID matching nothing is rejected | ID Resolution |
| INT-6 | Agent sessions from BOTH layouts fold into the row by default | Agent Fold-In |
| INT-7 | `agents::0` reports the root session alone | Agent Fold-In |
| INT-8 | `agents::` outside `0`/`1` is rejected | Input Validation |
| INT-9 | `session_ids::` with no non-empty ID is rejected | Input Validation |
| INT-10 | Golden pricing example — TTL split, unknown-TTL fallback, synthetic skip, compaction | Pricing |
| INT-11 | Bare `.cost` with no project for the cwd exits 2 | Exit Codes |
| INT-12 | `path::` anchors default resolution to another project | Default Resolution |
| INT-13 | Duplicate requests for one conversation collapse to one row | ID Resolution |
| INT-14 | A session ID duplicated across projects resolves to the richest copy | ID Resolution |

## Test Coverage Summary

- Default Resolution: 2 tests (INT-1, INT-12)
- Multi-Conversation: 1 test (INT-2)
- ID Resolution: 5 tests (INT-3, INT-4, INT-5, INT-13, INT-14)
- Agent Fold-In: 2 tests (INT-6, INT-7)
- Input Validation: 2 tests (INT-8, INT-9)
- Pricing: 1 test (INT-10)
- Exit Codes: 1 test (INT-11)

## Test Cases

---

### INT-1: Bare `.cost` reports the current conversation as one row without TOTAL

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .cost
```

**Expected behavior:**
- Fixture: one project matching the cwd with one 4-entry session (2 assistant calls, unpriced `claude-test` model)
- Output starts with the exact 11-column header (`Conversation Agents Req Input Output CacheR CacheW Total MaxCtx Compact Cost`); exactly one body row, byte-exact at the command's column widths; no `TOTAL` row for a single conversation
- The unpriced-model footnote `note: no pricing for model 'claude-test' — its tokens are excluded from Cost` and the trailing `Cost: estimated at API list prices (...)` line are both present
- Exit code: 0
- **Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md)

---

### INT-2: Multiple `session_ids::` produce rows in request order plus a TOTAL row

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .cost session_ids::bbbb2222-...,aaaa1111-...
```

**Expected behavior:**
- Fixture: two conversations in two different projects with distinct token totals, requested B-then-A (the reverse of both alphabetical and creation order)
- Rows appear in request order (B, then A), followed by a byte-exact TOTAL row summing every additive column (Req 3, Input 300, Output 120, Total 420) with `—` for the non-additive MaxCtx
- Exactly 3 body rows (two conversations + TOTAL)
- Exit code: 0
- **Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md), [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

---

### INT-3: A unique session ID prefix resolves to its conversation

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .cost session_ids::aaaa1111
```

**Expected behavior:**
- Fixture: one session whose 36-char ID starts with the requested 8-char prefix; no other session shares it
- The conversation resolves and renders as exactly one body row labeled by its short ID
- Exit code: 0
- **Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md), [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

---

### INT-4: An ambiguous session ID prefix is rejected naming every match

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .cost session_ids::aaaa
```

**Expected behavior:**
- Fixture: two sessions sharing the `aaaa` prefix in one project
- stderr contains `ambiguous session ID prefix 'aaaa': matches <id1>, <id2>` with the full IDs in sorted order — the command never silently picks one
- No table output on stdout
- Exit code: 1
- **Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md), [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

---

### INT-5: A session ID matching nothing is rejected

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .cost session_ids::zzzz
```

**Expected behavior:**
- Fixture: valid but empty storage (a `projects/` directory with no sessions)
- stderr contains `Session not found: zzzz`, naming the failing request rather than producing an empty table
- No table output on stdout
- Exit code: 1
- **Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md), [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

---

### INT-6: Agent sessions from BOTH layouts fold into the row by default

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .cost session_ids::cccc3333-...
```

**Expected behavior:**
- Fixture: one root session (2 calls) plus two hierarchical agents (`{uuid}/subagents/*.jsonl`, 2 calls each) plus one flat agent (`agent-*.jsonl` associated via first-entry `sessionId`, 2 calls) — per the [Session Family invariant](../../../../docs/invariant/002_session_family.md), both layouts in one family
- The single body row is byte-exact with `Agents` 3, `Req` 8, `Input` 80, `Output` 40, `Total` 120 — every agent session's usage folded into the root's row
- Exit code: 0
- **Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md), [param/40_agents.md](../../../../docs/cli/param/40_agents.md)

---

### INT-7: `agents::0` reports the root session alone

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .cost session_ids::cccc3333-... agents::0
```

**Expected behavior:**
- Fixture: identical family to INT-6 (root + 2 hierarchical agents + 1 flat agent)
- The single body row is byte-exact with `Agents` 0, `Req` 2, `Input` 20, `Output` 10, `Total` 30 — only the root's own usage, proving the opt-out actually excludes the same files INT-6 proved are included
- Exit code: 0
- **Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md), [param/40_agents.md](../../../../docs/cli/param/40_agents.md)

---

### INT-8: `agents::` outside `0`/`1` is rejected

**Command:**
```
clg .cost agents::2
```

**Expected behavior:**
- Run with no storage environment at all — passing proves validation precedes storage access (Finding #010 convention: a default does not exempt a parameter from explicit range checking)
- stderr contains `agents must be 0 or 1`; no table output on stdout
- Exit code: 1
- **Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md), [param/40_agents.md](../../../../docs/cli/param/40_agents.md)

---

### INT-9: `session_ids::` with no non-empty ID is rejected

**Command:**
```
clg .cost session_ids::,
```

**Expected behavior:**
- A value that trims/splits to nothing (a lone comma) is an argument error; run with no storage environment at all — passing proves the emptiness check precedes storage access
- stderr contains `session_ids must contain at least one session ID`; no table output on stdout
- Exit code: 1
- **Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md), [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

---

### INT-10: Golden pricing example — TTL split, unknown-TTL fallback, synthetic skip, compaction

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .cost session_ids::dddd4444-...
```

**Expected behavior:**
- Fixture: one conversation exercising every scanner special case at once — a haiku-4-5 call (1M in / 200k out / 3M read / 500k write split 400k 5m + 100k 1h), a second haiku call whose 200k write carries no `cache_creation` TTL breakdown (billed at the 5m rate, the API default TTL), a sonnet-5 call (500k in / 100k out), a `<synthetic>` entry (999,999 input tokens — contributes nothing), and a `compact_boundary` marker
- Hand-computed cost at list prices: haiku-4-5 $1.00 + $1.00 + $0.30 + $0.50 + $0.20 + $0.25 = $3.25; sonnet-5 $1.00 + $1.00 = $2.00; row total **$5.25**
- The body row is byte-exact: `Req` 3 (synthetic excluded), `Input` 1,500,000, `Output` 300,000, `CacheR` 3,000,000, `CacheW` 700,000, `Total` 5,500,000, `MaxCtx` 4,500,000 (largest single call), `Compact` 1, `Cost` $5.25
- No `note:` footnote (every model priced); the `Cost: estimated at API list prices (...)` line is present
- Exit code: 0
- **Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md)

---

### INT-11: Bare `.cost` with no project for the cwd exits 2

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .cost
```

**Expected behavior:**
- Fixture: valid empty storage; cwd matches no project — the "not found = usage error" convention shared with `.usage`/`.rollup`
- stderr contains `No project found for current directory`
- Exit code: 2
- **Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md)

---

### INT-12: `path::` anchors default resolution to another project

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .cost path::/elsewhere/project
```

**Expected behavior:**
- Fixture: one project (not the cwd) with one session; the command runs from an unrelated directory
- With `session_ids::` omitted, the reported conversation is the most recent session of the project owning `path::` — exactly one body row, labeled by that session's short ID
- Exit code: 0
- **Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md), [param/09_path.md](../../../../docs/cli/param/09_path.md)

---

### INT-13: Duplicate requests for one conversation collapse to one row

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .cost session_ids::<id>,<id>
```

**Expected behavior:**
- Fixture: one session requested twice by full ID
- Exactly one body row and no TOTAL row — the duplicate collapses instead of double-counting usage or triggering the multi-row TOTAL
- Exit code: 0
- **Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md), [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)

---

### INT-14: A session ID duplicated across projects resolves to the richest copy

**Command:**
```
CLAUDE_STORAGE_ROOT=/tmp/test-fixture clg .cost session_ids::ffff6666-...
```

**Expected behavior:**
- Fixture: the same session ID in two project directories (git-worktree-style forked history) — a 1-call copy and a 3-call copy
- The `Fix(BUG-528)` tie-break applies: the copy with the greatest entry count is reported — byte-exact row with the 3-call copy's numbers (`Req` 3, `Input` 30, `Total` 45); exactly one body row, never both, never the poorer copy
- Exit code: 0
- **Source:** [command/15_cost.md](../../../../docs/cli/command/15_cost.md), [param/39_session_ids.md](../../../../docs/cli/param/39_session_ids.md)
