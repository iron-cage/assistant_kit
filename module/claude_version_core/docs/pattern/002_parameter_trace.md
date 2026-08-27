# Pattern: Parameter Trace

**Status**: Implemented | **Since**: 1.4.1

### Scope

- **Purpose**: Document the unconditional stderr parameter-trace convention applied to every public mutating function in `claude_version_core` and the shared `claude_core::settings_io` module.
- **Responsibility**: Describe the problem, trace line format, rationale, and non-goals of the parameter-trace design.
- **In Scope**: The 11 traced functions, trace line format, stderr-only placement, unconditional (ungated) emission.
- **Out of Scope**: Leveled/structured logging (see Non-Goals), the sibling `claude_version` CLI crate's own pre-existing `eprintln!` diagnostics (an unrelated, already-existing idiom this pattern does not touch).

## Problem

Before this pattern, none of `claude_version_core`'s public mutating functions emitted any diagnostic trace. A state-changing operation could run — writing `settings.json`, chmod'ing the versions directory, running the installer — with zero observable trail if something went wrong or behaved unexpectedly. Debugging required re-running with ad hoc instrumentation added by hand.

## Solution

Every one of the 11 public mutating functions emits exactly one unconditional `eprintln!` call as its first statement, before any other logic runs. The trace line names the function and every one of its parameters:

```rust
pub fn set_setting( path : &Path, key : &str, raw_value : &str ) -> Result< StoredAs, io::Error >
{
  eprintln!( "set_setting(path={path:?}, key={key:?}, raw_value={raw_value:?})" );
  // ... existing logic unchanged
}
```

- Always stderr, never stdout — preserves stdout's pipeline-composability for commands that print machine-parseable output
- Unconditional — no verbosity flag suppresses it; fires on every call regardless of outcome
- Placed as the literal first statement — fires before the function can fail, short-circuit, or branch, so the trace is present even when the function errors partway through

## Function Coverage

| # | Function | File | Parameters traced |
|---|----------|------|--------------------|
| 1 | `hot_swap_binary` | `src/version.rs` | (none) |
| 2 | `purge_stale_versions` | `src/version.rs` | `versions_dir`, `keep` |
| 3 | `unlock_versions_dir` | `src/version.rs` | (none) |
| 4 | `lock_version` | `src/version.rs` | `is_latest`, `resolved` |
| 5 | `perform_install` | `src/version.rs` | `resolved`, `is_latest` |
| 6 | `store_preferred_version` | `src/version.rs` | `spec`, `resolved`, `is_latest` |
| 7 | `set_setting` | `claude_core/src/settings_io.rs` | `path`, `key`, `raw_value` |
| 8 | `remove_setting` | `claude_core/src/settings_io.rs` | `path`, `key` |
| 9 | `set_env_var` | `claude_core/src/settings_io.rs` | `path`, `key`, `value` |
| 10 | `remove_env_var` | `claude_core/src/settings_io.rs` | `path`, `key` |
| 11 | `unlock_settings_for_install` | `src/version.rs` | (none — promoted from `fn` to `pub fn` per BUG-017) |

7 of the 11 functions live in this crate's own `src/version.rs`. The other 4 live in `claude_core::settings_io` — a shared L0 primitive also used by `claude_profile` and `claude_runner_core` for their own settings/prefs files. `claude_version_core::settings_io` is a thin re-export shim over the same functions (`pub use claude_core::settings_io::*;`), so tracing the `claude_core` copy covers every caller, including this crate's own.

## Applicability

This pattern applies to any function that:
- Mutates persistent state outside the process (filesystem, `$HOME`, a spawned subprocess)
- Is reachable from a user-facing command (via the `claude_version` CLI layer)

It does not apply to private helper functions (e.g. `atomic_write`) — every private helper is only ever reached through an already-traced public function, so tracing it too would duplicate the same call's visibility without adding information about which external action initiated it.

**Exception — private helpers promoted to public:** when a private helper is the sole guardian of a critical invariant (e.g. lifting all installer-blocking lock keys before the installer runs), the testability requirement overrides the private-helper exemption. Promote to `pub fn`, add the trace as the first statement, and add integration tests. This was applied to `unlock_settings_for_install()` per BUG-017: a private helper that could not be tested from `tests/` had no regression tripwire for key-set divergence with `lock_version()`.

## Consequences

**Benefits:**
- Every mutating call leaves a diagnostic trail on stderr, even when it fails partway through
- No new dependency — plain `eprintln!`
- Deterministically testable: 6 of the 11 sites (`purge_stale_versions`, `unlock_settings_for_install`, and the 4 `settings_io` functions) get a static source-guard test (`include_str!`/`extract_fn_body`, asserting the trace is the function's first statement, no runtime capture); the other 5 (`hot_swap_binary`, `unlock_versions_dir`, `lock_version`, `perform_install`, `store_preferred_version` — real `$HOME`/`PATH`/network, no injectable seam) get CLI-subprocess-isolated tests capturing real stderr output

**Costs:**
- Every traced call now prints to stderr unconditionally — any script or tooling that treats non-empty stderr as a failure signal must account for this
- No verbosity gating — cannot be silenced per-invocation; if trace volume ever becomes unmanageable, the fallback path is introducing a real logging crate (`log`/`tracing`) with level filtering

## Non-Goals

- Leveled or structured output (log levels, JSON structured logs) — no concrete need identified beyond "leaves an unconditional trace"; would also contradict the unconditional requirement
- Verbosity/opt-out gating — same rationale
- Introducing a logging crate (`log`/`tracing`) — deferred; this pattern uses zero new dependencies

## Sources

- `../../src/version.rs` — 7 of the 11 traced functions
- `../../../claude_core/src/settings_io.rs` — 4 of the 11 traced functions

## Tests

| File | Relationship |
|------|-------------|
| `../../tests/version_test.rs` | Structural guards for `purge_stale_versions` and `unlock_settings_for_install` |
| `../../../claude_core/tests/settings_io_test.rs` | Structural guards for `set_setting`/`remove_setting`/`set_env_var`/`remove_env_var` |

## See Also

| File | Relationship |
|------|-------------|
| [claude_version/tests/cli/mutation_version_guard_test.rs](../../../claude_version/tests/cli/mutation_version_guard_test.rs) | CLI-layer subprocess-isolated stderr assertions for the 5 no-injectable-seam functions |
