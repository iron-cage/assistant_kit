# Add `version::` Parameter to `.version.guard`

## Goal

`.version.guard` accepts an optional `version::` parameter that overrides the stored
preferred version for a single invocation. Running `cm .version.guard version::2.1.72`
checks whether `2.1.72` is installed and installs it if not — without altering the stored
preference. Without this param, guard behaviour is unchanged (reads stored preference).
Unknown parameters must be rejected with exit 1, not silently ignored.

## In Scope

- `module/claude_manager/src/main.rs`: add `version_arg()` to `.version.guard` registration
- `module/claude_manager/src/commands.rs`:
  - `version_guard_routine` extracts optional `version::` and passes it down
  - `guard_once` accepts optional `version_override: Option<String>` and uses it
    instead of `read_preferred_version()` when `Some`
- `module/claude_manager/tests/integration/mutation_commands_test.rs`:
  new tests covering `version::` accepted, unknown param rejected, dry-run with override
- `docs/cli/commands.md`: Command :: 5 params table — add `version::` row
- `docs/cli/params.md`: `version::` applicable-commands — add `.version.guard`
- `spec.md`: FR-05 or new FR for guard version-override behaviour

## Out of Scope

- Persisting the `version::` override to stored preference
- `.version.guard` validating that the given version string actually exists on npm
- Changing `interval::` or `force::` semantics

## Description

When the user ran:

```
cm .version.guard interval::30 version::2.1.72
```

the `version::2.1.72` token was silently ignored because `.version.guard` is registered
without `version_arg()`, so the SemanticAnalyzer rejects it with an error — meaning the
user's invocation should have exited 1. The desired behaviour is:

1. `version::` is a recognised optional parameter for `.version.guard`.
2. When provided, it overrides the stored preference for that run only.
3. Unknown parameters (e.g. `bogus::x`) continue to exit 1 with a clear error.

Implementation note: `guard_once` currently calls `read_preferred_version()` to get the
target version. Add a `version_override` argument; if `Some(ver)`, skip `read_preferred_version()`
and use `ver` directly as both `spec` and `resolved`.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note constraints affecting file layout or code style.
2. **Write Test Matrix** — populate every row before opening any test file.
3. **Write failing tests** — implement test cases from the Test Matrix. Confirm failures.
4. **Register param** — add `version_arg()` to `.version.guard` in `build_registry()`.
5. **Update handler** — extend `version_guard_routine` and `guard_once` for override.
6. **Update docs** — `commands.md` (Command :: 5) and `params.md` (`version::` row).
7. **Update spec** — add or extend FR for guard version-override.
8. **Green state** — `w3 .test l::3` must pass with zero failures and zero warnings.
9. **Refactor if needed** — no function over 50 lines, no duplication.
10. **Walk Validation Checklist** — every answer must be YES.
11. **Update task status** — set status in `task/readme.md`, move file to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behaviour |
|---|---------------|-------------------|--------------------|
| T01 | `.version.guard version::9.9.9 dry::1` | version override + dry-run | exits 0; output mentions `9.9.9` |
| T02 | `.version.guard bogus::x` | unknown param | exits 1; stderr contains `unknown parameter` |
| T03 | `.version.guard version::9.9.9 force::1 dry::1` | override + force + dry | exits 0; output mentions `9.9.9` |
| T04 | `.version.guard` (no args) | no override — reads stored pref | exits 0; no mention of `9.9.9` |
| T05 | `.version.guard version::` (empty value) | empty version string | exits 1; error mentions version |

## Acceptance Criteria

- `version_arg()` added to `.version.guard` registration in `build_registry()`
- `guard_once` uses `version_override` instead of `read_preferred_version()` when `Some`
- `cm .version.guard version::9.9.9 dry::1` exits 0 and mentions `9.9.9` in output
- `cm .version.guard bogus::x` exits 1 with unknown-parameter error
- `docs/cli/commands.md` Command :: 5 params table includes `version::`
- `docs/cli/params.md` `version::` row applicable-commands includes `.version.guard`
- `w3 .test l::3` passes clean

## Validation Checklist

Desired answer for every question is YES.

**Registration**
- [ ] Is `version_arg()` present in `.version.guard` entry of `build_registry()`?
- [ ] Does `cm .version.guard bogus::x` exit 1?

**Override behaviour**
- [ ] Does `cm .version.guard version::9.9.9 dry::1` exit 0?
- [ ] Does dry-run output mention `9.9.9`?
- [ ] Is stored preference unchanged after running with `version::` override?

**Documentation**
- [ ] Does Command :: 5 params table in `commands.md` include `version::`?
- [ ] Does `version::` row in `params.md` list `.version.guard` as applicable?
- [ ] Is `spec.md` updated with guard version-override behaviour?

**Out of Scope confirmation**
- [ ] Is stored preference NOT updated when `version::` override is used?
- [ ] Does `interval::` and `force::` semantics remain unchanged?

## Validation Procedure

### Measurements

**M1 — Registration check**
`grep -n "version_guard" module/claude_manager/src/main.rs` — must show `version_arg()` in args vec.

**M2 — Override dry-run**
`cargo run -p claude_manager -- .version.guard version::9.9.9 dry::1`
Expected: exit 0, stdout contains `9.9.9`.

**M3 — Unknown param rejection**
`cargo run -p claude_manager -- .version.guard bogus::x`
Expected: exit 1, stderr contains `unknown parameter` (case-insensitive).

### Anti-faking checks

**AF1 — Test specificity**
T01 must assert `stdout.contains("9.9.9")`, not just `assert_exit(&out, 0)`.

**AF2 — No stored-pref mutation**
Dry-run with `version::` override must not write to `~/.claude/settings.json`.

## Outcomes

**Completed 2026-03-24.**

- `version_arg()` added to `.version.guard` in `build_registry()` (`main.rs:117`)
- `version_guard_routine` extracts `version_override: Option<String>` and validates it via `validate_version_spec()`
- `guard_once` signature changed to `fn guard_once(dry, force, version_override: Option<&str>)`
- Override branch: resolves alias via `resolve_version_spec()`, bypasses `read_preferred_version()` entirely
- 4 new tests: TC-411..TC-414 (all pass)
- `spec.md`: Command Inventory row 5 updated (5 params); Parameter Inventory row 1 updated (2 cmds); FR-18 extended; FR-21 added
- `docs/cli/commands.md`: row 5 count 4→5; Command :: 5 syntax + params table updated with `version::` row and override example
- `docs/cli/params.md`: row 1 count 1→2; Parameter :: 1 description and Commands updated
- `docs/cli/parameter_interactions.md`: Scope `version::` line added
- L3 (nextest + doc tests + clippy): 1056/1056 pass, 0 warnings
