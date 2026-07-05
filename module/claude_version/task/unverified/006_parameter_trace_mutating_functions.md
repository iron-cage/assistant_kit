# Task 006: Instrument Parameter Trace on All Mutating Functions

## Execution State

- **Executor Type:** any
- **filed_by:** dev
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ❓ (Unverified)
- **closes:** Q-01
- **unit_type:** module
- **unit:** lib/yrd_core/assistant_kit/claude_version/module/claude_version_core
- **validated_by:** null
- **validation_date:** null

## Goal

Add an unconditional `eprintln!` trace line at the entry point of every public mutating function in `claude_version_core`, printing the function name and its resolved parameters, per the user's explicit standing directive that "all commands changing someing must have paramter trace." **Motivated:** today only a handful of call sites emit ad hoc diagnostic `eprintln!`s (`version.rs:263,274,448`, `lib.rs:243,263,275,293`), while the 10 public mutating functions enumerated below emit no trace at all — a state-changing operation can run with zero diagnostic trail. **Observable:** running any operation that reaches one of the 10 functions (`.version.install`, `.version.guard`, `.settings.set`/`.settings.unset` equivalents, `.config set`/`.config unset`) prints one stderr line naming the function and its parameter values before the mutation occurs. **Scoped:** exactly the 10 public mutating functions in `version.rs` and `settings_io.rs` listed in In Scope; private helpers are excluded (see Out of Scope). **Testable:** `verb/test_only trace` plus new assertions in `claude_version_core/tests/version_test.rs` and `settings_io_test.rs` capturing stderr for each of the 10 functions.

## In Scope

- `module/claude_version_core/src/version.rs` — add an entry-point trace line to: `hot_swap_binary`, `purge_stale_versions`, `unlock_versions_dir`, `lock_version`, `perform_install`, `store_preferred_version`
- `module/claude_version_core/src/settings_io.rs` — add an entry-point trace line to: `set_setting`, `remove_setting`, `set_env_var`, `remove_env_var`
- Trace format: a single unconditional `eprintln!` per call, naming the function and its parameter values, consistent across all 10 sites (per `task/decisions.md` Q-01's assumed mechanism)
- New `module/claude_version/docs/pattern/002_parameter_trace.md` documenting the resulting convention (see Acceptance Criteria — created once the code exists, not before)
- Tests asserting stderr trace output for a representative call of each of the 10 functions

## Out of Scope

- Private/internal helper functions not part of the public API surface (e.g., `atomic_write`) — every private helper is only ever reached through an already-traced public function, so tracing it too would duplicate the same call's visibility without adding information about which external action initiated it
- `claude_version` (CLI) crate's own command handlers — a trace at the `claude_version_core` public API boundary already covers every mutating call regardless of which CLI command invoked it; a second trace layer at the CLI layer would be pure duplication
- Introducing a logging crate (`log`/`tracing`) — deferred; `task/decisions.md` Q-01 documents this as the fallback path if unconditional `eprintln!` proves insufficient (trace volume unmanageable, or structured/filterable output later needed)
- Structured or leveled output (log levels, JSON structured logs, opt-out/verbosity gating) — YAGNI; no concrete need identified beyond "leaves an unconditional trace," and gating would contradict the user's literal "must have" (unconditional) phrasing

## Null Hypothesis

Do nothing — rely on the existing ad hoc `eprintln!` calls at a few call sites and no tracing elsewhere.

**Disproof:** The user's own standing directive states this requirement in imperative terms for every state-mutating command, not as a suggestion. This is a concrete, already-issued instruction, not speculative hardening. `task/decisions.md` Q-01 records the assumed mechanism (unconditional `eprintln!`, no new dependency) based on the existing ungated `eprintln!` idiom already present in this codebase and the absence of any `log`/`tracing` crate dependency in either crate's `Cargo.toml`. Disproof would require the user retracting the directive, or Q-01 being invalidated in favor of a real logging crate.

## Requirements

- Every one of the 10 listed public mutating functions MUST emit exactly one stderr trace line on every call, unconditionally (no verbosity flag suppresses it)
- The trace line MUST name the function and include every parameter's value (not just a bare function name)
- Trace output MUST go to stderr, never stdout (preserves stdout pipeline-composability of existing commands, e.g. `.runtime_files | xargs`)
- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the Work Procedure below.

- All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
- Test Matrix populated before any test code
- All Test Matrix cases implemented as failing tests before implementation
- Minimum code to satisfy Test Matrix — no features beyond requirements
- `verb/test` passes with zero failures and zero warnings
- No function exceeds 50 lines; no duplication; public items have `///` doc comments
- Independent validation passes via MAAV (this rulebook's Verification Gate) — never self-verified
- Task state updated to 🎯 on validation pass; file moved from `task/unverified/` to `task/` root
- `task/decisions.md` Q-01 state updated to reflect this task closing it (via the task's own completion, not a separate DECIDE/CONFIRM MAAV cycle — out of this task's scope)

### Work Procedure

1. Read all 10 target function signatures in full (`version.rs`, `settings_io.rs`) to confirm exact parameter lists
2. Write the 10 Test Matrix rows below as failing tests capturing stderr output
3. Add one `eprintln!` trace line at the top of each of the 10 functions, naming the function and its parameters
4. Run `verb/test_only trace` until all 10 rows pass
5. Create `module/claude_version/docs/pattern/002_parameter_trace.md` (Level 2, `pattern/` type) documenting the convention: which functions trace, trace format, rationale, and explicit non-goals (no leveling, no new dependency)
6. Run full `verb/test` (all crates) and confirm zero failures, zero warnings

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | call `hot_swap_binary(...)` | any | stderr contains `hot_swap_binary` trace line with its parameters |
| T02 | call `purge_stale_versions(...)` | any | stderr contains `purge_stale_versions` trace line with its parameters |
| T03 | call `unlock_versions_dir(...)` | any | stderr contains `unlock_versions_dir` trace line with its parameters |
| T04 | call `lock_version(...)` | any | stderr contains `lock_version` trace line with its parameters |
| T05 | call `perform_install(...)` | any | stderr contains `perform_install` trace line with its parameters |
| T06 | call `store_preferred_version(...)` | any | stderr contains `store_preferred_version` trace line with its parameters |
| T07 | call `set_setting(...)` | any | stderr contains `set_setting` trace line with its parameters |
| T08 | call `remove_setting(...)` | any | stderr contains `remove_setting` trace line with its parameters |
| T09 | call `set_env_var(...)` | any | stderr contains `set_env_var` trace line with its parameters |
| T10 | call `remove_env_var(...)` | any | stderr contains `remove_env_var` trace line with its parameters |

## Acceptance Criteria

- All 10 Test Matrix rows have a corresponding passing test
- All 10 functions' trace lines appear on stderr, never stdout
- `module/claude_version/docs/pattern/002_parameter_trace.md` exists at Level 2 documentation completeness
- `verb/test` (full suite) passes with zero failures and zero warnings

## Validation

**Execution:** The procedure for walking this section is defined in `validation.rulebook.md`. The executor does NOT self-validate — an independent validator performs the walk after EXEC_COMPLETE transition (⚙️ → 🔎).

### Checklist

Desired answer for every question is YES.

**Trace coverage**
- [ ] C1 — Do all 6 `version.rs` target functions emit a trace line?
- [ ] C2 — Do all 4 `settings_io.rs` target functions emit a trace line?
- [ ] C3 — Does every trace line include the function's parameter values (not just its name)?

**Out of Scope confirmation**
- [ ] C4 — Is `atomic_write` (or any other private helper) still untraced (no duplicate tracing added)?
- [ ] C5 — Is `claude_version` (CLI crate) free of any new trace lines added by this task?

### Measurements

- [ ] M1 — trace site count: `grep -c 'eprintln!' module/claude_version_core/src/version.rs module/claude_version_core/src/settings_io.rs` → increased by exactly 10 from this session's baseline

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures
- [ ] I2 — compiler clean: `RUSTFLAGS="-D warnings" cargo check` → 0 warnings

### Anti-faking checks

- [ ] AF1 — stdout purity: a traced command's stdout (e.g. `.version.install` piped) contains zero trace-line bytes — trace must be stderr-only
- [ ] AF2 — no new dependency: `grep -c 'log\|tracing' module/claude_version_core/Cargo.toml module/claude_version/Cargo.toml` → 0 (confirms Q-01's no-new-dependency assumption held)

## Related Documentation

- `module/claude_version/task/decisions.md` — Q-01 (Parameter-Trace Instrumentation Mechanism), closed by this task
- `module/claude_version/docs/pattern/002_parameter_trace.md` — created by this task (Work Procedure step 5)
- `module/claude_version_core/src/version.rs` — 6 of the 10 target functions
- `module/claude_version_core/src/settings_io.rs` — 4 of the 10 target functions

**Closes:** Q-01

## History

- **[2026-07-05]** `CREATED` — Instrument unconditional stderr parameter trace on all 10 public mutating functions in `claude_version_core`, per the user's standing directive that all state-changing commands must have a parameter trace.
