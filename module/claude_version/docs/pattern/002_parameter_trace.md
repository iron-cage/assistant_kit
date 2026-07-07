# Pattern: Parameter Trace

### Scope

- **Purpose**: Document the unconditional stderr parameter-trace convention applied to every public mutating function in `claude_version_core` and the shared `claude_core::settings_io` module.
- **Responsibility**: Describe the problem, trace line format, rationale, and non-goals of the parameter-trace design.
- **In Scope**: The 10 traced functions, trace line format, stderr-only placement, unconditional (ungated) emission.
- **Out of Scope**: Leveled/structured logging (see Non-Goals), the sibling `claude_version` CLI crate's own pre-existing `eprintln!` diagnostics (an unrelated, already-existing idiom this pattern does not touch).

### Problem

Before this pattern, none of `claude_version_core`'s public mutating functions emitted any diagnostic trace. A state-changing operation could run — writing `settings.json`, chmod'ing the versions directory, running the installer — with zero observable trail if something went wrong or behaved unexpectedly. Debugging required re-running with ad hoc instrumentation added by hand.

### Solution

Every one of the 10 public mutating functions emits exactly one unconditional `eprintln!` call as its first statement, before any other logic runs. The trace line names the function and every one of its parameters:

```rust
pub fn set_setting( path : &Path, key : &str, raw_value : &str ) -> Result< StoredAs, io::Error >
{
  eprintln!( "set_setting(path={path:?}, key={key:?}, raw_value={raw_value:?})" );
  // ... existing logic unchanged
}
```

- Always stderr, never stdout — preserves stdout's pipeline-composability for commands that print machine-parseable output (e.g. `.runtime_files | xargs`)
- Unconditional — no verbosity flag suppresses it; fires on every call regardless of outcome
- Placed as the literal first statement — fires before the function can fail, short-circuit, or branch, so the trace is present even when the function errors partway through

### Function Coverage

| # | Function | File | Parameters traced |
|---|----------|------|--------------------|
| 1 | `hot_swap_binary` | `claude_version_core/src/version.rs` | (none) |
| 2 | `purge_stale_versions` | `claude_version_core/src/version.rs` | `versions_dir`, `keep` |
| 3 | `unlock_versions_dir` | `claude_version_core/src/version.rs` | (none) |
| 4 | `lock_version` | `claude_version_core/src/version.rs` | `is_latest`, `resolved` |
| 5 | `perform_install` | `claude_version_core/src/version.rs` | `resolved`, `is_latest` |
| 6 | `store_preferred_version` | `claude_version_core/src/version.rs` | `spec`, `resolved`, `is_latest` |
| 7 | `set_setting` | `claude_core/src/settings_io.rs` | `path`, `key`, `raw_value` |
| 8 | `remove_setting` | `claude_core/src/settings_io.rs` | `path`, `key` |
| 9 | `set_env_var` | `claude_core/src/settings_io.rs` | `path`, `key`, `value` |
| 10 | `remove_env_var` | `claude_core/src/settings_io.rs` | `path`, `key` |

4 of the 10 functions live in `claude_core::settings_io` — a shared L0 primitive also used by `claude_profile` and `claude_runner_core` for their own settings/prefs files. `claude_version_core::settings_io` is a thin re-export shim over the same functions (`pub use claude_core::settings_io::*;`), so tracing the `claude_core` copy covers every caller, including `claude_version_core`'s own.

### Applicability

This pattern applies to any function that:
- Mutates persistent state outside the process (filesystem, `$HOME`, a spawned subprocess)
- Is reachable from a user-facing command (`.version.install`, `.version.guard`)

It does not apply to private helper functions (e.g. `atomic_write`) — every private helper is only ever reached through an already-traced public function, so tracing it too would duplicate the same call's visibility without adding information about which external action initiated it.

### Consequences

**Benefits:**
- Every mutating call leaves a diagnostic trail on stderr, even when it fails partway through
- No new dependency — plain `eprintln!`, consistent with the sibling `claude_version` CLI crate's own pre-existing (unrelated) diagnostic idiom
- Deterministically testable: 5 of the 10 sites (`purge_stale_versions` plus the 4 `settings_io` functions) have an injectable parameter and get a static source-guard test (`include_str!`, asserting the trace is the function's first statement, no runtime capture); the other 5 (`hot_swap_binary`, `unlock_versions_dir`, `lock_version`, `perform_install`, `store_preferred_version` — real `$HOME`/`PATH`/network, no injectable seam) get CLI-subprocess-isolated tests capturing real stderr output

**Costs:**
- Every traced call now prints to stderr unconditionally — any script or tooling that treats non-empty stderr as a failure signal must account for this
- No verbosity gating — cannot be silenced per-invocation; if trace volume ever becomes unmanageable, the fallback path is introducing a real logging crate (`log`/`tracing`) with level filtering

### Non-Goals

- Leveled or structured output (log levels, JSON structured logs) — no concrete need identified beyond "leaves an unconditional trace"; would also contradict the unconditional requirement
- Verbosity/opt-out gating — same rationale
- Introducing a logging crate (`log`/`tracing`) — deferred; this pattern uses zero new dependencies (see `task/decisions.md` Q-01 for the fallback path if ever needed)

### Features

| File | Relationship |
|------|-------------|
| [feature/001_version_management.md](../feature/001_version_management.md) | `.version.install`/`.version.guard` reach 6 of the 10 traced functions |

### Sources

| File | Relationship |
|------|-------------|
| `../../../claude_version_core/src/version.rs` | 6 of the 10 traced functions |
| `../../../claude_core/src/settings_io.rs` | 4 of the 10 traced functions |

### Provenance

| Source | Notes |
|--------|-------|
| `../../../../task/claude_version/decisions.md` | Q-01 (Parameter-Trace Instrumentation Mechanism), closed by task 313 |

### Tests

| File | Relationship |
|------|-------------|
| [../../../claude_version_core/tests/version_test.rs](../../../claude_version_core/tests/version_test.rs) | Structural guard for `purge_stale_versions` |
| [../../../claude_core/tests/settings_io_test.rs](../../../claude_core/tests/settings_io_test.rs) | Structural guards for `set_setting`/`remove_setting`/`set_env_var`/`remove_env_var` |
| [../../tests/cli/mutation_version_guard_test.rs](../../tests/cli/mutation_version_guard_test.rs) | Subprocess-isolated stderr assertions for `hot_swap_binary`/`unlock_versions_dir`/`lock_version`/`perform_install`/`store_preferred_version` |
