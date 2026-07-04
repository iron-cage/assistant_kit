# Model Select Command — Implementation

## Execution State

- **Executor Type:** any
- **filed_by:** agent
- **actor:** agent
- **started_at:** 2026-07-02
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **dir:** src/ + module/claude_runner_core/src/
- **validated_by:** null
- **validation_date:** null

## Goal

Users running `clr run/ask/isolated/refresh` cannot currently pin a specific subprocess model across sessions without setting environment variables or modifying workspace configuration. `ISOLATED_DEFAULT_MODEL` is a compile-time constant; there is no user-facing override path. This task exposes `~/.clr/prefs.json` as that override, enabling persistent model selection without environment changes.

Implement the `.model.select` command in `claude_profile` and the `~/.clr/prefs.json` reader integration in `claude_runner_core`, as specified in `docs/feature/069_model_select_command.md` and `docs/cli/command/007_model.md` (Command #20).

Observable end-state: `clp .model.select id::claude-opus-4-8` creates `~/.clr/prefs.json` with `{"subprocess_model":"claude-opus-4-8"}`; `clp .model.select` without args prints `model.select: claude-opus-4-8`; `clp .model.select reset::1` removes the key idempotently; `clp .model.select id:: ` exits 1; `clp .model.select id::X reset::1` exits 1 with `mutually exclusive` in stderr; `clr run` (or `isolated`) picks up the pinned model from `prefs.json` instead of `ISOLATED_DEFAULT_MODEL`; `w3 .test level::3` passes in both `claude_profile` and `claude_runner_core` with zero failures.

## In Scope

- `src/commands/model_select.rs` — new file; `.model.select` command handler; get mode (no `id::`, `reset::0`): read `~/.clr/prefs.json` → extract `subprocess_model` → print `model.select: VALUE` or `model.select: (unset)`; set mode (`id::VALUE`): validate non-empty → read prefs JSON (or `{}`) → set `subprocess_model` → write back → print `model.select: VALUE (pinned)`; reset mode (`reset::1`): read prefs JSON → remove `subprocess_model` key → write back → print `model.select: (reset to default)` (idempotent when file absent); mutual exclusion: `id::` + `reset::1` → exit 1 stderr `model.select: id:: and reset::1 are mutually exclusive`; empty `id::` → exit 1 stderr `id:: must be a non-empty model ID`; `format::json` in get mode renders `{"subprocess_model":"VALUE"}` or `{"subprocess_model":null}`
- `src/registry.rs` — register `.model.select` command with `id::` (param 064), `reset::` (param 066), `format::` parameters; add to help command group
- `module/claude_runner_core/src/isolated.rs` — in `run_isolated_ext()` (or equivalent model resolution site), before applying `ISOLATED_DEFAULT_MODEL`: attempt to read `~/.clr/prefs.json`; parse `subprocess_model` field; if present and non-empty, use this model ID; if absent, parsing error, or empty, fall back to `ISOLATED_DEFAULT_MODEL` unchanged
- `tests/cli/model_select_test.rs` — new test file in `claude_profile/tests/`; implement all 12 test cases from `tests/docs/cli/command/20_model_select.md` (IT-01..IT-12) and all 12 from `tests/docs/feature/069_model_select_command.md` (FT-01..FT-12); use isolated temp `~/.clr/` directory for each test
- `module/claude_runner_core/tests/isolated_test.rs` — extend to cover `prefs.json` model override: test that when `~/.clr/prefs.json` exists with a `subprocess_model` key, `run_isolated_ext()` selects that model instead of `ISOLATED_DEFAULT_MODEL`; test absent `prefs.json` still uses `ISOLATED_DEFAULT_MODEL`

## Out of Scope

- Model ID validation against the live API (no-op at write time; API rejects at use time)
- Integration with `imodel::` touch/refresh subprocess model selection (intentionally separate — see Feature 026)
- `.models` command implementation (→ Task 007)
- Stale ID fix in `map_model_shorthand()` / `resolve_model()` (→ Task 009)

## Work Procedure

1. Read `src/commands/model.rs` (`.model` command) to understand the `read_prefs_json()` / `write_prefs_json()` helper pattern for settings.json; adapt it for `~/.clr/prefs.json`
2. Read `module/claude_runner_core/src/isolated.rs` to locate the `ISOLATED_DEFAULT_MODEL` constant usage site and the model resolution logic
3. Create `src/commands/model_select.rs`: implement the three modes (get/set/reset) and two error cases (empty `id::`, mutual exclusion); use a local `read_clr_prefs()` / `write_clr_prefs()` helper that reads/writes `~/.clr/prefs.json`; create `~/.clr/` dir if absent on write
4. Register `.model.select` in `src/registry.rs` with params `id::`, `reset::`, `format::`; add to help group alongside `.model`
5. In `module/claude_runner_core/src/isolated.rs`, add `read_subprocess_model_pref() -> Option<String>` that reads `~/.clr/prefs.json`; call it in the model resolution path before falling back to `ISOLATED_DEFAULT_MODEL`
6. Write failing test cases in `tests/cli/model_select_test.rs` covering IT-01..IT-12 and FT-01..FT-12; each test sets up an isolated temp directory via env override for the home path; seed `prefs.json` fixtures as needed for IT-05, IT-08
7. Extend `module/claude_runner_core/tests/isolated_test.rs` to cover prefs.json model override (2 new tests: pinned model used; absent prefs falls back to default)
8. From `module/claude_profile/`, run `w3 .test level::1`; fix all failures; iterate until green
9. From `module/claude_runner_core/`, run `w3 .test level::1`; fix all failures
10. From `module/claude_profile/`, run `w3 .test level::3`; from `module/claude_runner_core/`, run `w3 .test level::3`; zero clippy warnings required in both crates

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `clp .model.select` (no prefs.json) | Get: absent file | Stdout `model.select: (unset)`; exit 0 |
| `clp .model.select` (prefs.json has `subprocess_model`) | Get: value present | Stdout `model.select: claude-opus-4-8`; exit 0 |
| `clp .model.select id::claude-opus-4-8` | Set mode | `prefs.json` contains key; stdout has `(pinned)`; exit 0 |
| `clp .model.select id::claude-sonnet-5` | Set: sonnet | `prefs.json` contains `"subprocess_model":"claude-sonnet-5"`; exit 0 |
| `clp .model.select reset::1` (with seeded prefs.json) | Reset: removes key | `subprocess_model` absent; `other_key` preserved; exit 0 |
| `clp .model.select reset::1` (no prefs.json) | Reset: idempotent | Stdout `(reset to default)`; exit 0; no error |
| `clp .model.select id::claude-opus-4-8` (no prefs.json) | Set: creates file | `prefs.json` created with key; exit 0 |
| `clp .model.select id::claude-opus-4-8` (prefs.json has `other_key`) | Set: preserves others | Both `subprocess_model` and `other_key` present; exit 0 |
| `clp .model.select id::claude-opus-4-8 reset::1` | Mutual exclusion | Exit 1; stderr contains `mutually exclusive` |
| `clp .model.select format::json` (prefs.json set) | JSON format | Stdout `{"subprocess_model":"claude-opus-4-8"}`; exit 0 |
| `clp .help` | Help registration | Output contains `.model.select`; exit 0 |
| `clp .model.select id::` | Empty id validation | Exit 1; stderr mentions non-empty required |
| `run_isolated_ext()` with prefs.json pinned | clr integration | Model flag = pinned value, not `ISOLATED_DEFAULT_MODEL` |
| `run_isolated_ext()` with absent prefs.json | clr fallback | Model flag = `ISOLATED_DEFAULT_MODEL` |

## Related Documentation

- `docs/feature/069_model_select_command.md` — full feature specification (AC-01..AC-12)
- `docs/cli/command/007_model.md` — CLI command spec including Command #20 `.model.select`
- `docs/schema/008_clr_prefs_json.md` — `~/.clr/prefs.json` schema
- `docs/cli/param/064_id.md` — `id::` parameter specification
- `docs/cli/param/066_reset.md` — `reset::` parameter specification
- `tests/docs/cli/command/20_model_select.md` — IT-01..IT-12 integration test cases
- `tests/docs/feature/069_model_select_command.md` — FT-01..FT-12 feature test cases

## History

- **[2026-07-02]** `CREATED` — Implement `.model.select` command (get/set/reset modes) and `~/.clr/prefs.json` subprocess model preference reader in `claude_runner_core`.

## Verification Findings

**Round 1 — FAILED (MOST Goal Quality: M dimension; Implementation Readiness). Resolved before re-verify.**

Finding 1 (M): Goal section explained *what* to build but provided no *why* — no user need, no problem with current state, no consequence of absence.

Fix applied: Added motivation paragraph explaining that `ISOLATED_DEFAULT_MODEL` is compile-time constant with no user-facing override path, and that `~/.clr/prefs.json` fills that gap.

Finding 2 (IR): Work Procedure steps 8-10 referenced "in `claude_profile`" / "in `claude_runner_core`" without stating the directory path needed to run `w3 .test`. Executor must not infer directory.

Fix applied: Steps 8-10 now reference `module/claude_profile/` and `module/claude_runner_core/` as explicit directory contexts for the `w3 .test` invocations.

## Verification Record

**Round 2 — PASSED (all 4 dimensions). Date: 2026-07-02.**

| Dimension | Result | Agent |
|-----------|--------|-------|
| Scope Coherence | PASS | ae9028fa4658fc535 (Round 1) |
| MOST Goal Quality | PASS | a082e8ea2612f6f44 (M re-verify Round 2) |
| Value / YAGNI | PASS | a6742e94b83e4c953 (Round 1) |
| Implementation Readiness | PASS | aa53cef9781df5633 (IR re-verify Round 2) |
