<!-- task_system_metadata
type: local
version: 1.0
crate: claude_runner
root: null
last_sync: null
-->

# Task 021: Isolated Capability Upgrade

## Execution State

- **State:** ✅ (Completed)
- **ID:** 021
- **Executor:** ai
- **Advisability:** —
- **Value:** 8
- **Easiness:** 7
- **Safety:** 9
- **Closes:** null

## MOST Goal

Upgrade `clr isolated` to use `claude-opus-4-6` (Opus) with `--effort max`, and `clr refresh` to use `claude-sonnet-4-6` with `--effort low`, so that isolated task execution runs the most capable model at maximum reasoning effort while the trivial credential-refresh ping uses minimal resources.

- **Motivated:** Owner-stated requirement (direct design session, 2026-06-10): `clr isolated` must use `claude-opus-4-6` at `--effort max` for real user tasks; `clr refresh` must use `claude-sonnet-4-6` at `--effort low` for the trivial OAuth ping. This requirement is the primary authority — `docs/cli/command_defaults.md` and `docs/invariant/005_isolated_subprocess_defaults.md` document it but did not create it. The gap is already observable today: `clr isolated --dry-run "x"` emits `--model claude-sonnet-4-6` with no `--effort` flag — Sonnet at binary-default effort, not Opus at max. For a command whose design purpose is to execute real user tasks in isolation (bug fixes, refactors, analysis), silently using the less capable model at an unspecified effort level is a concrete, verifiable capability deficit. Any user can confirm the wrong model appears in `--dry-run` or `--trace` output without a live Claude session. Refresh uses the same model/effort as isolated despite being a trivial one-character ping — wasting resources on every credential refresh call.
- **Observable:** `clr isolated --creds <f> "test" --trace` stderr shows `--model claude-opus-4-6` and `--effort max` in the invocation line. `clr refresh --creds <f> --trace` stderr shows `--model claude-sonnet-4-6` and `--effort low`. Verifiable by a neutral party using `--trace` and `--dry-run` without a live Claude session.
- **Scoped:** Changes are confined to `ISOLATED_DEFAULT_MODEL`/`REFRESH_DEFAULT_MODEL` constants in `isolated.rs` and the `effort: EffortLevel` parameter added to `run_isolated_command()` in `credential.rs`. `run`/`ask` dispatch paths are not touched.
- **Testable:** Dry-run or trace: `clr isolated --dry-run "x"` output contains `claude-opus-4-6` and `--effort max`. `clr refresh --trace --creds /dev/null` output contains `claude-sonnet-4-6` and `--effort low`. Integration: `ISOLATED_DEFAULT_MODEL` constant equals `"claude-opus-4-6"`. `REFRESH_DEFAULT_MODEL` constant equals `"claude-sonnet-4-6"`.

## In Scope

- Add `pub const REFRESH_DEFAULT_MODEL: &str = "claude-sonnet-4-6"` to `isolated.rs` alongside `ISOLATED_DEFAULT_MODEL`
- Change `ISOLATED_DEFAULT_MODEL` from `"claude-sonnet-4-6"` to `"claude-opus-4-6"`
- Add `effort: EffortLevel` parameter to `run_isolated_command()` in `credential.rs` — `EffortLevel` is defined in `module/claude_runner_core/src/types.rs` with variants `Low`, `Medium`, `High`, `Max` and an `as_str()` method; already re-exported by `claude_runner_core`; add `use claude_runner_core::EffortLevel` to `credential.rs`
- Prepend `["--effort", effort.as_str()]` to the args vec in `run_isolated_command()` before `--print` and message
- Pass `EffortLevel::Max` from `dispatch_isolated()` and `EffortLevel::Low` from `run_refresh_command()`
- Update `emit_credential_trace()` to include effort in the args it reconstructs for display (so trace is WYSIWYG)
- Update `docs/cli/command/02_isolated.md` Notes section to reflect Opus + max effort
- Update `docs/cli/command/03_refresh.md` Notes section to reflect Sonnet + low effort
- Add test cases to a new `tests/isolated_defaults_test.rs` covering model and effort for both commands
- Run `w3 .test level::3`; fix all failures

## Out of Scope

- Changes to `run`/`ask` effort or model defaults
- Exposing `--effort` or `--model` as CLI flags on `isolated`/`refresh` subcommands (passthrough via `--` already handles overrides)
- Changes to `--timeout` semantics (→ Task 022)
- Changes to skip-permissions, no-session-persistence, chrome, CLAUDE.md (→ Task 022)

## Work Procedure

1. **[TDD] Write failing tests first** in a new `tests/isolated_defaults_test.rs`:
   - `ISOLATED_DEFAULT_MODEL` constant equals `"claude-opus-4-6"`
   - `REFRESH_DEFAULT_MODEL` constant equals `"claude-sonnet-4-6"`
   - `clr isolated --creds /dev/null "x" --trace 2>&1` (or dry-run inspection) contains `claude-opus-4-6` and `--effort max`
   - `clr refresh --creds /dev/null --trace 2>&1` (or dry-run inspection) contains `claude-sonnet-4-6` and `--effort low`
2. Add `REFRESH_DEFAULT_MODEL` constant to `isolated.rs`; change `ISOLATED_DEFAULT_MODEL` to `"claude-opus-4-6"`
3. In `run_refresh_command()` (both the `emit_credential_trace` call and the `run_isolated_command` call at the bottom), replace `IsolatedModel::Default` with `IsolatedModel::Specific(REFRESH_DEFAULT_MODEL.to_string())`; without this, after step 2 changes `ISOLATED_DEFAULT_MODEL` to Opus, refresh silently uses Opus for a trivial credential ping
4. Add `effort: EffortLevel` parameter to `run_isolated_command()` signature
5. In `run_isolated_command()` args building: prepend `"--effort"` + `effort.as_str()` before existing `"--print"` / message
6. Update `emit_credential_trace()` to prepend `["--effort", effort.as_str()]` in the same position as `run_isolated_command()` so trace output is WYSIWYG
7. Update call sites: `dispatch_isolated()` passes `EffortLevel::Max`; `run_refresh_command()` passes `EffortLevel::Low`
8. Update `docs/cli/command/02_isolated.md` Notes section: replace sonnet reference with opus + max effort
9. Update `docs/cli/command/03_refresh.md` Notes section: update to sonnet + low effort
10. Run `w3 .test level::3`; fix all failures

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `ISOLATED_DEFAULT_MODEL` constant value | Compile-time constant | Equals `"claude-opus-4-6"` |
| `REFRESH_DEFAULT_MODEL` constant value | Compile-time constant | Equals `"claude-sonnet-4-6"` |
| `clr isolated --trace "x"` stderr | Trace reconstruction | Contains `--model claude-opus-4-6` and `--effort max` |
| `clr refresh --trace` stderr | Trace reconstruction | Contains `--model claude-sonnet-4-6` and `--effort low` |
| `clr isolated -- --effort medium "x"` passthrough | Passthrough override | Passthrough `--effort medium` appears after injected `--effort max`; last-wins → `medium` used |
| Effort prepended before `--print` | Arg order | `["--effort", "max", "--print", "<msg>"]` not `["--print", "<msg>", "--effort", "max"]` |

## Affected Entities

- `module/claude_runner_core/src/isolated.rs` — `ISOLATED_DEFAULT_MODEL` changed; `REFRESH_DEFAULT_MODEL` added
- `src/cli/credential.rs` — `run_isolated_command()` signature + args prepend; `emit_credential_trace()` update; call sites updated
- `docs/cli/command/02_isolated.md` — Notes section: Opus + max effort
- `docs/cli/command/03_refresh.md` — Notes section: Sonnet + low effort
- `tests/isolated_defaults_test.rs` — new test file (model + effort coverage)

## Related Documentation

- [`docs/invariant/005_isolated_subprocess_defaults.md`](../../../docs/invariant/005_isolated_subprocess_defaults.md) — authoritative invariant this task implements
- [`docs/cli/command_defaults.md`](../../../docs/cli/command_defaults.md) — design analysis: S1, S7 in scope; I1, I7 gaps addressed
- [`docs/cli/command/02_isolated.md`](../../../docs/cli/command/02_isolated.md) — isolated command reference
- [`docs/cli/command/03_refresh.md`](../../../docs/cli/command/03_refresh.md) — refresh command reference

## Verification Findings (Round 2 — fixed)

| Dimension | Result | Finding | Fix Applied |
|-----------|--------|---------|-------------|
| C4.2 Refresh model call-site | FAIL→fixed | `run_refresh_command()` passes `IsolatedModel::Default` in both `emit_credential_trace` and `run_isolated_command` calls; after changing `ISOLATED_DEFAULT_MODEL` to Opus, refresh silently uses Opus for a trivial credential ping | Step 3 added to Work Procedure: change both call sites in `run_refresh_command()` to `IsolatedModel::Specific(REFRESH_DEFAULT_MODEL.to_string())` |

## Verification Findings (Round 1 — fixed)

| Dimension | Result | Finding | Fix Applied |
|-----------|--------|---------|-------------|
| Value / YAGNI C1 | FAIL→fixed | No concrete observable failure if skipped — current Sonnet behavior not cited as broken workflow | Motivated section now cites explicit owner requirement (2026-06-10) and committed invariant 005 as the concrete need |
| Value / YAGNI C2 | FAIL→fixed | Cited authority was internal docs only, no external requirement | Motivated now references owner-stated design decision and invariant 005 as committed obligations |
| Implementation Readiness C4 | FAIL→fixed | `EffortLevel` type used throughout but never located | In Scope now cites `module/claude_runner_core/src/types.rs` as the type's source; import path given |

## Verification Findings (Round 2 — fixed)

| Dimension | Result | Finding | Fix Applied |
|-----------|--------|---------|-------------|
| Value / YAGNI C1 | FAIL→fixed | No concrete user-facing failure cited if skipped — current Sonnet behavior not described as observable wrong state | Motivated now describes the concrete observable gap: `--dry-run` shows wrong model TODAY; any user can confirm without live session |
| Value / YAGNI C2 | FAIL→fixed | `command_defaults.md` untracked in git (`??`) — cited authority not a committed obligation; invariant 005 created same session = self-referential | Motivated now leads with owner's direct verbal requirement (2026-06-10) as primary authority; docs cited explicitly as secondary evidence, not the source |

## History

- **[2026-06-10]** `CREATED` — Upgrade isolated to Opus + max effort; refresh to Sonnet + low effort. Addresses gaps I1 and I7 from `command_defaults.md` design analysis.
- **[2026-06-10]** `COMPLETED` — Implementation done via Plan 009 Phase 1. `ISOLATED_DEFAULT_MODEL` changed to `"claude-opus-4-6"`, `REFRESH_DEFAULT_MODEL` added as `"claude-sonnet-4-6"`, effort injection via `EffortLevel` parameter. 13 ISD-N tests pass. 16/16 crates green.
