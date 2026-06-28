# CLI Documentation Collection Normalization

## Execution State

- **Executor Type:** any
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🎯 (Verified)
- **closes:** null
- **dir:** docs/cli/
- **validated_by:** N/A
- **validation_date:** N/A

## Goal

Normalize all `docs/cli/` collection instances to consistent structural conventions — standardized H1 format (`# EntityType :: N. Name`), Algorithm sections for all 11 commands, Referenced Type/Groups sections for all 24 params, and canonical section ordering within each entity type. **Why now:** the `cli_doc_des.rulebook.md` audit exposed 28 structural violations (missing Algorithm sections, absent Referenced sections, inconsistent H1 formats, wrong section ordering) that block `doc_graph.yml` cross-reference validation and prevent the `rulebook_cli_doc` skill from passing its compliance checks — any new command or param addition would inherit the inconsistent patterns. Observable end-state: all 61 collection instances (11 commands, 24 params, 3 formats, 5 user stories, 5 param groups, 13 types) follow their entity type's canonical structure. Administrative task: documentation reorganization plus minimal test alignment (2 assertion fixes, 10 clippy `doc_markdown` fixes). Testable: `w3 .test level::3` passes with 655 tests and 0 warnings.

## In Scope

**Documentation normalization (docs/cli/):**
- `command/01-11*.md`: Algorithm sections added (2-5 steps each, derived from source); Referenced section reordering (Groups → Formats → Params → Stories)
- `param/01-24*.md`: Referenced Parameter Groups section added for 14 command-specific params; Referenced Type Boolean added for 3 new boolean params (19, 23, 24); section ordering normalized to Pattern A (Type → Commands → Groups → Stories) for 7 files
- `format/01-03*.md`: H1 normalized from `# Format: X` to `# Format :: N. X`
- `user_story/001-005*.md`: H1 normalized from `# Name` to `# User Story :: N. Name`
- `type/13_topic_name.md → 12_topic_name.md`, `type/14_strategy_type.md → 13_strategy_type.md`: renumbered
- `cli/002_dictionary.md → cli/dictionary.md`, `cli/006_workflows.md → cli/workflows.md`: numeric prefix removed
- `cli/readme.md`: Completion Matrix label corrected (L5 "Test Detail Complete", 100%)

**Test alignment (tests/):**
- `tests/docs/cli/format/01-03*.md`: H1 aligned with docs counterparts
- `cli_user_story_audit_session_history_test.rs`: RWS-2 restored to `show_tokens` per spec
- `cli_user_story_query_storage_programmatically_test.rs`: RWS-1 assertion made case-insensitive
- `cli_param_show_stat_test.rs`, `cli_param_show_tokens_test.rs`, `cli_param_show_tree_test.rs`, `cli_cmd_projects_summary_test.rs`: clippy `doc_markdown` backtick fixes

## Out of Scope

- Source code behavior changes (no functional changes to any CLI command routine)
- New feature implementation (all normalized params already implemented)
- Documentation outside `docs/cli/` (feature/, operation/ untouched)
- Test coverage expansion (only alignment fixes, no new scenarios added)
- param_group/ and type/ content changes (only renumbering/reordering, no semantic edits)

## Requirements

- All work must strictly adhere to `cli_doc_des.rulebook.md` (NxN Relationship Matrix, H1 format, Required sections)
- All work must strictly adhere to `doc_des.rulebook.md` (Collection, Progressive Documentation)

## Delivery Requirements

Administrative task — test-related and implementation constraints skipped per `tsk.rulebook.md`.

- All work must strictly adhere to all applicable rulebooks
- `w3 .test level::3` passes with zero failures and zero warnings
- No orphaned cross-references (all Referenced sections point to existing targets)
- Task state updated to ✅ on validation pass; file moved to `task/completed/`

## Acceptance Criteria

- AC-1: All 11 command files contain an `**Algorithm (N steps):**` section
- AC-2: All 24 param files contain `### Referenced Type` and `### Referenced Parameter Groups` sections
- AC-3: All format files use H1 format `# Format :: N. Name`
- AC-4: All user_story files use H1 format `# User Story :: N. Name`
- AC-5: Command files use section order: Groups → Formats → Params → Stories
- AC-6: Param files use section order: Type → Commands → Groups → Stories (Pattern A)
- AC-7: L3 verification (655 tests, 0 clippy warnings) passes cleanly

## Validation

**Execution:** Independent validator per `validation.rulebook.md`. Administrative task — `Validated By` and `Validation Date` are N/A.

### Checklist

**H1 format consistency**
- [ ] C1 — Do all 3 `docs/cli/format/*.md` files match `^# Format :: [0-9]+\.`?
- [ ] C2 — Do all 5 `docs/cli/user_story/*.md` files match `^# User Story :: [0-9]+\.`?

**Required sections**
- [ ] C3 — Do all 11 `docs/cli/command/*.md` files contain `**Algorithm (`?
- [ ] C4 — Do all 24 `docs/cli/param/*.md` files contain `### Referenced Type`?
- [ ] C5 — Do all 24 `docs/cli/param/*.md` files contain `### Referenced Parameter Groups`?

**Section ordering**
- [ ] C6 — In command files, does `### Referenced Parameter Groups` appear before `### Referenced Parameters`?
- [ ] C7 — In param files, does `### Referenced Commands` appear before `### Referenced Parameter Groups`?

**Out of Scope confirmation**
- [ ] C8 — Are `docs/feature/` and `docs/operation/` files unmodified by this task?

### Measurements

- [ ] M1 — command Algorithm count: `grep -l 'Algorithm (' docs/cli/command/[0-9]*.md | wc -l` → 11
- [ ] M2 — param Referenced Type count: `grep -l 'Referenced Type' docs/cli/param/[0-9]*.md | wc -l` → 24
- [ ] M3 — test count: `cargo nextest run --all-features 2>&1 | grep 'tests run'` → 655 passed

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check` → 0 warnings

### Anti-faking checks

- [ ] AF1 — H1 negative: `grep -rn '^# Format:' docs/cli/format/ | wc -l` → 0 (old format absent)
- [ ] AF2 — H1 negative: `grep -rn '^# [A-Z][a-z]' docs/cli/user_story/ | grep -v '# User Story ::' | wc -l` → 0

## Related Documentation

- `docs/cli/readme.md` — CLI documentation hub with Completion Matrix
- `docs/cli/command/readme.md` — Command collection index
- `docs/cli/param/readme.md` — Parameter collection index
- `docs/cli/format/readme.md` — Format collection index
- `docs/cli/user_story/readme.md` — User story collection index
- `docs/cli/type/readme.md` — Type collection index
- `docs/entity.md` — Master doc entity and instance tables
- `tests/docs/cli/format/01_markdown.md`, `02_json.md`, `03_text.md` — Format test mirrors
- `tests/docs/cli/user_story/01_audit_session_history.md` — User story test spec (RWS-2 fix)

## History

- **[2026-06-28]** `CREATED` — Task filed. Goal: normalize CLI doc collection instance structure for consistency.
- **[2026-06-28]** `VERIFY_PASS` — Verification Gate passed (4/4 dimensions PASS). Moved to 🎯 Verified.

## Verification Record

| Dimension | Result | Agent ID |
|-----------|--------|----------|
| Scope Coherence | PASS | a01d76d867734e3d2 |
| MOST Goal Quality | PASS | a0f3763266849013f |
| Value / YAGNI | PASS | a356c4aeb394cf554 |
| Implementation Readiness | PASS | aa5b7b14cb3e0069d |

**Note (Implementation Readiness):** Minor observation — AC-2/C5 state "all 24 param files" for Referenced Parameter Groups; 14 were newly added (command-specific params), the other 10 already had the section via their group membership. All 24 verified present post-normalization.
