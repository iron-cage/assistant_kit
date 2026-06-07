<!-- task_system_metadata
type: local
version: 1.0
crate: claude_runner
root: null
last_sync: null
-->

# Task 022: Isolated Subprocess Correctness

## Execution State

- **State:** ✅ (Completed)
- **ID:** 022
- **Executor:** ai
- **Advisability:** —
- **Value:** 9
- **Easiness:** 6
- **Safety:** 9
- **Closes:** null

## MOST Goal

Fix five correctness gaps in the isolated/refresh subprocess so that: session persistence is suppressed (temp HOME is discarded anyway), permissions are not blocked, chrome is suppressed for refresh, `--timeout 0` means unlimited (matching run/ask), and a minimal CLAUDE.md is written to the temp HOME to prevent interactive blocking.

- **Motivated:** Five distinct correctness gaps exist in the current isolated/refresh implementation (issues I2–I6 from `command_defaults.md`): (1) Session files are written to a temp HOME that is unconditionally deleted — pure I/O waste. (2) `clr isolated "task"` blocks at every tool call because `--dangerously-skip-permissions` is not injected. (3) `clr refresh --trace` shows `--chrome` in the command line despite refresh being an HTTP-only OAuth ping. (4) `--timeout 0` kills the subprocess immediately on the first 50ms poll — opposite of run/ask semantics where 0 = unlimited. (5) The subprocess has no `~/.claude/CLAUDE.md` in the temp HOME, so it may ask clarifying questions or request confirmation and block forever.
- **Observable:** (a) `clr isolated --dry-run "x"` invocation line includes `--no-session-persistence` and `--dangerously-skip-permissions`. (b) `clr refresh --trace` invocation line includes `--no-chrome` and excludes `--chrome`. (c) `clr isolated --timeout 0 --trace "x"` does NOT kill the subprocess at 50ms — it runs until the subprocess exits or is killed by the OS. (d) A CLAUDE.md file exists at `<temp_home>/.claude/CLAUDE.md` at subprocess spawn time (readable via trace or added assertions). Verifiable by neutral party without a live Claude session.
- **Scoped:** Changes confined to `credential.rs::run_isolated_command()` (flags S3, S4, S5), `isolated.rs::run_isolated()` (S2 timeout fix, S6 CLAUDE.md write). `run`/`ask` execution paths are not touched. The `ClaudeCommand` builder and `cred_parse.rs` are not changed.
- **Testable:** Dry-run or trace assertions on injected flags. Test for timeout=0: spawn a fake script that sleeps 10s; with `--timeout 0`, the script is NOT killed within 2s (it runs to natural completion). Test for CLAUDE.md: after `run_isolated()` writes creds, assert the CLAUDE.md file content matches the required string before subprocess spawn.

## In Scope

- **S2 — Fix timeout=0 semantics** in `isolated.rs::run_isolated()`: when `timeout_secs == 0`, skip the deadline loop entirely and wait for the subprocess to exit naturally (no `try_wait` timeout poll)
- **S3 — Inject `--no-session-persistence`** in `credential.rs::run_isolated_command()`: prepend `"--no-session-persistence"` to args vec before `--print` and message; applies to both isolated and refresh
- **S4 — Suppress `--chrome` for refresh** in `credential.rs::run_refresh_command()`: prepend `"--no-chrome"` to the args passed to `run_isolated_command()`; isolated retains the `ClaudeCommand` default chrome
- **S5 — Inject `--dangerously-skip-permissions` for isolated** in `credential.rs::run_isolated_command()`: prepend `"--dangerously-skip-permissions"` when `message.is_some()` (real task present); omit for refresh (which calls with `message = Some(".")` — but skip-perms is harmless for refresh; simplest rule: always inject when message is present)
- **S6 — Write CLAUDE.md** in `isolated.rs::run_isolated()`: after creating `claude_dir` and writing `.credentials.json`, write the required CLAUDE.md content to `claude_dir/CLAUDE.md`
- Update `emit_credential_trace()` to include `--no-session-persistence`, `--dangerously-skip-permissions` (when applicable), and `--no-chrome` (for refresh) in the reconstructed arg list so trace is WYSIWYG
- Update `docs/cli/command/02_isolated.md` Notes section to document the new injections
- Update `docs/cli/command/03_refresh.md` Notes section to document the new injections
- Update `docs/cli/param/020_timeout.md` to document that `0` = unlimited (no watchdog)
- Add test cases in `tests/isolated_defaults_test.rs` (may extend the file created by Task 021, or create `tests/isolated_correctness_test.rs` if 021 was not yet done)
- Run `w3 .test level::3`; fix all failures

## Out of Scope

- Changes to `run`/`ask` timeout, skip-permissions, or persistence behavior
- Exposing `--no-session-persistence` or `--no-chrome` as top-level `isolated`/`refresh` CLI flags (injected automatically; passthrough `--` can still override)
- Model or effort changes (→ Task 021)
- Windows support (spawn loop and signal handling are Linux-only already)

## Work Procedure

1. **[TDD] Write failing tests first** — extend or create an `isolated_correctness_test.rs`:
   - `clr isolated --dry-run "x"` invocation contains `--no-session-persistence`
   - `clr isolated --dry-run "x"` invocation contains `--dangerously-skip-permissions`
   - `clr refresh --trace` invocation contains `--no-chrome` and does not contain `--chrome` (or `--chrome` appears before `--no-chrome` so last-wins applies)
   - `run_isolated()` with a fake subprocess (no creds needed): assert CLAUDE.md file at `<temp_home>/.claude/CLAUDE.md` exists and contains the required content before spawn
   - Timeout=0 behavior: fake script sleeps 5s; `timeout_secs=0` → subprocess runs to natural exit (not killed); test must complete within 10s
2. **S2**: in `run_isolated()`, change `let deadline = Instant::now() + Duration::from_secs(timeout_secs)` to be inside an `if timeout_secs > 0` branch; when `timeout_secs == 0`, run an unbounded `loop` that only calls `child.try_wait()` with 50ms sleep (no deadline check)
3. **S3**: in `run_isolated_command()`, push `"--no-session-persistence".to_string()` onto a leading prefix vec before the `--print` / message args
4. **S4**: in `run_refresh_command()`, include `"--no-chrome".to_string()` in `trace_args` and pass it as part of the args to `run_isolated_command()` (or as a leading prefix before passthrough_args)
5. **S5**: in `run_isolated_command()`, push `"--dangerously-skip-permissions".to_string()` onto the prefix vec when `message.is_some()`
6. **S6**: in `run_isolated()`, after `std::fs::write(&creds_path, credentials_json)`, write the CLAUDE.md content to `claude_dir.join("CLAUDE.md")`; use the exact content specified in `invariant/005_isolated_subprocess_defaults.md`
7. Update `emit_credential_trace()` to include S3/S4/S5 flags in its reconstructed args so dry-run/trace is WYSIWYG
8. Update `docs/cli/command/02_isolated.md` Notes: add injections (skip-permissions, no-session-persistence, CLAUDE.md)
9. Update `docs/cli/command/03_refresh.md` Notes: add injections (no-chrome, no-session-persistence, CLAUDE.md)
10. Update `docs/cli/param/020_timeout.md`: replace "A timeout of `0` causes immediate expiry" with "A timeout of `0` disables the watchdog — the subprocess runs until it exits naturally"
11. Run `w3 .test level::3`; fix all failures

## Test Matrix

| Input Scenario | Config Under Test | Expected Behavior |
|----------------|-------------------|-------------------|
| `clr isolated --dry-run "x"` | `--no-session-persistence` injection | Invocation line includes `--no-session-persistence` |
| `clr isolated --dry-run "x"` | `--dangerously-skip-permissions` injection | Invocation line includes `--dangerously-skip-permissions` |
| `clr refresh --trace` | `--no-chrome` injection | Invocation line includes `--no-chrome`; `--chrome` absent or appears before `--no-chrome` |
| `run_isolated()` CLAUDE.md write | CLAUDE.md provisioning (S6) | File at `<temp_home>/.claude/CLAUDE.md` exists and contains required content before subprocess spawn |
| `run_isolated()` timeout=0 | S2 — unlimited semantics | Subprocess with 5s sleep is NOT killed within 2s; exits naturally |
| `run_isolated()` timeout=30 (default) | Deadline active | Subprocess exceeding 30s is killed and returns `TimeoutWithOutput` |
| Passthrough `-- --session-persistence` | Override injected no-persist | Last-wins: session-persistence re-enabled by passthrough |
| Message absent (`clr isolated --creds f`) | S5 — skip-perms condition | `--dangerously-skip-permissions` NOT injected when no message |

## Affected Entities

- `module/claude_runner_core/src/isolated.rs` — `run_isolated()`: S2 timeout fix, S6 CLAUDE.md write
- `src/cli/credential.rs` — `run_isolated_command()`: S3 no-session-persistence, S5 skip-permissions; `run_refresh_command()`: S4 no-chrome; `emit_credential_trace()`: WYSIWYG update
- `docs/cli/command/02_isolated.md` — Notes section: new injections documented
- `docs/cli/command/03_refresh.md` — Notes section: new injections documented
- `docs/cli/param/020_timeout.md` — timeout=0 semantics corrected
- `tests/isolated_correctness_test.rs` (or extension of `isolated_defaults_test.rs`) — correctness coverage

## Related Documentation

- [`docs/invariant/005_isolated_subprocess_defaults.md`](../../../docs/invariant/005_isolated_subprocess_defaults.md) — authoritative invariant this task implements
- [`docs/cli/command_defaults.md`](../../../docs/cli/command_defaults.md) — design analysis: S2, S3, S4, S5, S6 in scope; I2, I3, I4, I5, I6 gaps addressed
- [`docs/cli/command/02_isolated.md`](../../../docs/cli/command/02_isolated.md) — isolated command reference (Notes section updated by this task)
- [`docs/cli/command/03_refresh.md`](../../../docs/cli/command/03_refresh.md) — refresh command reference (Notes section updated by this task)
- [`docs/cli/param/020_timeout.md`](../../../docs/cli/param/020_timeout.md) — timeout param doc (timeout=0 semantics corrected by this task)

## Verification Record

| Dimension | Result | Notes |
|-----------|--------|-------|
| Scope Coherence | PASS | Non-empty in/out scope with four concrete observable outcomes; exclusions are adjacent-but-distinct; no contradictions |
| MOST Goal Quality | PASS | M: five named correctness gaps with concrete harms; O: four independently verifiable checks without live session; S: bounded to two source files with explicit exclusions; T: specific test conditions in goal text |
| Value / YAGNI | PASS | Null hypothesis: five demonstrably broken behaviors in shipped code; committed need via I2–I6 and invariant 005; no speculative abstractions |
| Implementation Readiness | PASS | All 11 steps concrete; 8-row Test Matrix; source files verified to exist and match description; no unresolved external decisions |

Transition: ❓ → 🎯

## History

- **[2026-06-10]** `CREATED` — Fix five isolated/refresh subprocess correctness gaps: timeout-0 semantics (I2), session-persistence waste (I3), chrome overhead in refresh (I4), missing skip-permissions for tasks (I5), and missing CLAUDE.md causing interactive blocking (I6).
- **[2026-06-10]** `COMPLETED` — Implementation done via Plan 009 Phases 2+3. Timeout-0 fixed (`Option<Instant>`), `--no-session-persistence` injected, `--dangerously-skip-permissions` when message present, `--no-chrome` for refresh, CLAUDE.md provisioned. 13 ISD-N + 6 CT-N tests pass. 16/16 crates green.
