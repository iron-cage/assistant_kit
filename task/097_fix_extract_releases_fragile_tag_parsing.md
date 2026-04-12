# Fix extract_releases fragile literal-split tag parsing

## Execution State

- **Executor Type:** any
- **Actor:** null
- **Claimed At:** null
- **Status:** 🎯 (Available)

## Goal

Replace the fragile `split("\"tag_name\": \"v")` approach in `extract_releases` with a targeted field extraction that tolerates tags without a `v` prefix and is immune to the pattern appearing in release body text, verified by `w3 .test level::3`. (Motivated: GitHub may publish tags without the `v` prefix or use variations in JSON whitespace; a release whose body contains the literal string `"tag_name": "v` would also split incorrectly, yielding corrupted version strings; Observable: `extract_releases` correctly parses releases with and without `v` prefix; Scoped: only `extract_releases` in `commands.rs`; Testable: `cargo nextest run --test integration --features enabled -E 'test(releases)'`)

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_manager/src/commands.rs` — `extract_releases`: replace the `split("\"tag_name\": \"v")` approach with explicit field extraction by scanning for the `"tag_name"` key, then extracting the value regardless of whether it starts with `v`; strip a leading `v` if present
- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_manager/tests/integration/read_commands_test.rs` — add TC-470: `extract_releases` with a tag without `v` prefix parses correctly; TC-471: `extract_releases` where release body contains `"tag_name": "v` does not corrupt adjacent version fields

## Out of Scope

- Switching to a full JSON parser or adding serde dependency (no new dependencies)
- Changing `parse_json_string_value` or `extract_releases` network behaviour
- Changing surrogate pair handling (covered in TSK-096)

## Description

`extract_releases` parses GitHub API responses by splitting the entire JSON blob on the literal string `"\"tag_name\": \"v"`. This approach has two failure modes: (a) GitHub tags without a `v` prefix (e.g. `"2.1.0"` instead of `"v2.1.0"`) produce empty or garbled version strings because the split marker is never found; (b) a release whose body text contains the literal sequence `"tag_name": "v` would split the blob at the wrong boundary, corrupting all field extractions for subsequent releases.

The fix replaces the fragile split with explicit field scanning: search for each occurrence of the `"tag_name"` key in the JSON, extract the quoted value that follows (reusing `parse_json_string_value`), then strip a leading `v` if present. This approach is independent of the `v` prefix convention and immune to release body contamination.

No new dependencies are introduced; the fix stays within the existing hand-rolled extraction style.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   TDD: write failing tests before implementing; confirm they fail before fixing
-   No new crate dependencies; hand-rolled extraction must remain

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note code_design constraints on function length (≤50 lines) and no-duplication principle.
2. **Write Test Matrix** — populate all rows below before opening any test file.
3. **Write failing tests** — add TC-470, TC-471 calling `extract_releases` directly with crafted JSON strings; confirm failures.
4. **Read source** — read the full `extract_releases` function; understand the current split strategy and the `parse_json_string_value` helper it relies on.
5. **Implement** — replace the `split` approach: scan the JSON string for each occurrence of `"tag_name"`, extract the value after the colon (handling optional whitespace and quote), strip a leading `v`, then proceed with the existing field extraction for `published_at` and `body`.
6. **Green state** — `w3 .test level::3` must pass with zero failures and zero warnings.
7. **Submit for Validation** — trigger SUBMIT transition.
8. **Update task status** — on validation pass set ✅ in `task/readme.md`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Tag `"tag_name": "v2.1.0"` (normal, `v` prefix) | standard case unchanged | version = `"2.1.0"` |
| T02 | Tag `"tag_name": "2.1.0"` (no `v` prefix) | tag without `v` prefix | version = `"2.1.0"` |
| T03 | Release body contains `"tag_name": "v1.0.0"` literally | body contains the split marker | version field not polluted by body content |
| T04 | Multiple releases in one JSON blob | full extraction loop | all release objects parsed independently |
| T05 | Empty releases array `[]` | empty input | returns empty vec, no panic |

## Acceptance Criteria

-   `extract_releases` does not use `split("\"tag_name\": \"v")` as the primary parsing mechanism
-   Tags without `v` prefix parse correctly (version = bare semver)
-   A release body containing the old split marker does not corrupt adjacent fields
-   T01–T05 all pass
-   `w3 .test level::3` passes with zero failures

## Validation

### Checklist

Desired answer for every question is YES.

**Tag extraction**
- [ ] C1 — Is `split("\"tag_name\": \"v")` absent from `extract_releases`?
- [ ] C2 — Does `extract_releases` handle tags without `v` prefix (TC-T02 passes)?
- [ ] C3 — Does a release body containing the old split marker not corrupt the version field (TC-T03 passes)?
- [ ] C4 — Are all existing network-based release tests still passing?

**Out of Scope confirmation**
- [ ] C5 — Is `parse_json_string_value` unchanged?
- [ ] C6 — Are no new crate dependencies added to `Cargo.toml`?

### Measurements

- [ ] M1 — split pattern gone: `grep -c '"tag_name\\": \\"v"' src/commands.rs` → 0 (was: 1 — the fragile literal)
- [ ] M2 — no-prefix test passes: TC-470 `assert_eq!(releases[0].version, "2.1.0")` with no-`v` input passes (was: would have produced empty string or wrong version)

### Invariants

- [ ] I1 — test suite: `w3 .test level::3` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check --features enabled` → 0 warnings

### Anti-faking checks

- [ ] AF1 — tag_name field scan: `grep -n '"tag_name"' src/commands.rs` → at least 1 match (the new extraction searches for the key, not the key+value prefix)
- [ ] AF2 — v-prefix strip: `grep -n 'strip_prefix.*v\|starts_with.*v' src/commands.rs` → at least 1 match (optional `v` stripped after extraction)

## Outcomes

[Added upon task completion.]
