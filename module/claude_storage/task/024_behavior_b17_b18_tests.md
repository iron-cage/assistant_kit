# Add B17 and B18 behavior validation tests

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)

## Goal

Create `tests/behavior/b17_parentuuid_self_contained.rs` and `tests/behavior/b18_no_cross_session_links.rs` to validate hypotheses B17 and B18 documented in `docs/claude_code/001_session_behaviors.md` (Motivated: B17 and B18 are the foundational invariants that make cross-session conversation chain detection necessary — they prove that no intra-storage links exist between sessions — yet no tests validate them, leaving the hypothesis status as 🎯 Planned; Observable: both test files exist in `tests/behavior/`, `tests/behavior/mod.rs` declares both modules, `tests/behavior/readme.md` lists both files, and the tests run against real `~/.claude/` storage; Scoped: new test files only in `tests/behavior/`; no source changes; Testable: `w3 .test level::3` passes and `grep -c "b17\|b18" tests/behavior/mod.rs` returns 2).

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/behavior/b17_parentuuid_self_contained.rs`
  - Test: for a sample of sessions in `~/.claude/projects/`, parse all entries, collect all `uuid` values in the session, then verify every non-null `parentUuid` is either null (first entry) or references a `uuid` within the same session — no cross-file UUID references exist
  - Skip gracefully if `~/.claude/projects/` is absent
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/behavior/b18_no_cross_session_links.rs`
  - Test: for every root session file in a sample of projects, read the first non-zero-byte entry and assert `parentUuid` is `null` — new sessions always start with a null parent
  - Skip gracefully if `~/.claude/projects/` is absent
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/behavior/mod.rs`
  - Add `mod b17_parentuuid_self_contained;`
  - Add `mod b18_no_cross_session_links;`
  - Update the File Index doc comment table to include B17 and B18 rows
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/behavior/readme.md` (if it exists) — add rows for B17 and B18

## Out of Scope

- Implementing the conversation chain detection algorithm based on these behaviors (→ future task)
- Changes to `src/` production code
- Changes to any other test files
- Changing behavior status in `001_session_behaviors.md` (documentation was already updated)

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Tests must use real `~/.claude/` storage data — no synthetic fixtures (per no-mocking rule)
-   Tests must skip gracefully when `~/.claude/projects/` is absent (same pattern as existing B1–B16 tests)
-   Sample size: test a representative sample (up to 10 projects, up to 5 sessions per project) to keep test execution fast
-   B17 test must parse JSONL entries and extract `uuid` and `parentUuid` fields — use `serde_json::Value` for flexibility

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note test file format requirements and no-mocking policy.
2. **Read source** — Read `tests/behavior/mod.rs` to understand the file structure, helper functions (`find_sessions`, `find_projects`, `claude_projects_dir`), and the existing module declaration pattern.
3. **Read an existing behavior test** — Read `tests/behavior/b10_entry_threading.rs` to understand the JSONL parsing pattern and how entries are read.
4. **Read documentation** — Read `docs/claude_code/001_session_behaviors.md` §B17 and §B18 for the authoritative hypothesis statement and evidence references.
5. **Write b17 test** — Create `tests/behavior/b17_parentuuid_self_contained.rs` with test `it_parentuuid_never_crosses_session_boundary`. Confirm it fails or passes before proceeding (expected: PASS if real storage exists; SKIP if not).
6. **Write b18 test** — Create `tests/behavior/b18_no_cross_session_links.rs` with test `it_first_entry_parentuuid_is_null`. Confirm it fails or passes.
7. **Update mod.rs** — Add `mod b17_parentuuid_self_contained;` and `mod b18_no_cross_session_links;` and update the File Index table in the doc comment.
8. **Update readme.md** — Add B17 and B18 rows if `tests/behavior/readme.md` exists.
9. **Green state** — `w3 .test level::3` passes with zero failures and zero warnings.
10. **Walk Validation Checklist** — every answer must be YES.
11. **Update task status** — ✅ in `task/readme.md`, move to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `~/.claude/projects/` exists with ≥1 session | real storage | `it_parentuuid_never_crosses_session_boundary` passes |
| T02 | `~/.claude/projects/` absent | no storage | both tests skip (not fail) |
| T03 | session file with first entry | `parentUuid` field | `it_first_entry_parentuuid_is_null` finds `parentUuid == null` |

## Acceptance Criteria

- `tests/behavior/b17_parentuuid_self_contained.rs` exists with test `it_parentuuid_never_crosses_session_boundary`
- `tests/behavior/b18_no_cross_session_links.rs` exists with test `it_first_entry_parentuuid_is_null`
- `grep -c "b17\|b18" tests/behavior/mod.rs` returns 2
- Both tests pass with real `~/.claude/` storage present
- Both tests skip (not fail) when `~/.claude/projects/` is absent

## Validation

### Checklist

Desired answer for every question is YES.

**B17 test**
- [ ] C1 — Does `b17_parentuuid_self_contained.rs` exist and contain `it_parentuuid_never_crosses_session_boundary`?
- [ ] C2 — Does the test parse JSONL entries and verify all `parentUuid` values resolve within the same file?
- [ ] C3 — Does the test skip when `~/.claude/projects/` is absent?

**B18 test**
- [ ] C4 — Does `b18_no_cross_session_links.rs` exist and contain `it_first_entry_parentuuid_is_null`?
- [ ] C5 — Does the test read the first entry of each session and assert `parentUuid` is null?
- [ ] C6 — Does the test skip when `~/.claude/projects/` is absent?

**Registration**
- [ ] C7 — Does `tests/behavior/mod.rs` declare both `b17` and `b18` modules?
- [ ] C8 — Does the File Index doc comment in `mod.rs` list both B17 and B18?

**Out of Scope confirmation**
- [ ] C9 — Is `src/` unchanged?
- [ ] C10 — Are B1–B16 tests unchanged?

### Measurements

- [ ] M1 — test count: `w3 .test level::3 2>&1 | grep "test result"` → `test result: ok. 291 passed` (was: 289 + 2 new behavior tests; independent of tasks 021-023)

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --all-features` → 0 warnings

### Anti-faking checks

**AF1 — Verify both modules declared**
Check: `grep -c "b17_parentuuid_self_contained\|b18_no_cross_session_links" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/behavior/mod.rs`
Expected: 2. Why: catches partial implementation where one module was added but not the other.

**AF2 — Verify UUID cross-reference check in b17**
Check: `grep -n "parentUuid\|parent_uuid" /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_storage/tests/behavior/b17_parentuuid_self_contained.rs | grep -v "//"`
Expected: ≥ 2 matches. Why: ensures the test actually reads and checks `parentUuid` fields, not just file existence.

## Outcomes

[Added upon task completion]
