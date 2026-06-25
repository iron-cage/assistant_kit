# Feature: Trace Timestamp Prefix

### Scope

- **Purpose**: Replace the `[trace]` bracket prefix on all diagnostic trace output with a UTC timestamp, enabling time-based correlation between `clp` trace lines and watchdog logs.
- **Responsibility**: Every diagnostic line emitted when `trace::1` is active is prefixed with `trace_ts()`, a function in `claude_profile_core::account` that returns `"YYYY-MM-DD · HH:MM:SS · "` (UTC). All production source files that emit `trace::1` lines import and call `trace_ts()` in place of the former `"[trace] "` string literal.
- **In Scope**: `trace_ts()` implementation in `claude_profile_core/src/account.rs`; all 13 production files that emit `trace::1` diagnostic lines (see Sources); removal of all `"[trace] "` string literals from production `eprintln!`/`writeln!( std::io::stderr(), ...)` calls; help-text updates in `src/registry.rs`; test assertion updates across 12 test files.
- **Out of Scope**: Trace output content (message, label, account name, arguments) — only the prefix changes. The `trace::1` gate logic — `if trace { ... }` call-site guards are unchanged. The watchdog script or any external consumer of trace output.

### Design

All diagnostic trace lines previously began with the literal string `"[trace] "`. This prefix was statically embedded in every `eprintln!`/`writeln!` format string, making it impossible to correlate `clp` trace output with time-stamped logs (watchdog, cron, journald).

`trace_ts()` is added to `claude_profile_core/src/account.rs` immediately after `chrono_now_utc()`:

```rust
#[inline]
#[must_use]
pub fn trace_ts() -> String
{
  let utc = chrono_now_utc();
  format!( "{} · {} · ", &utc[..10], &utc[11..19] )
}
```

`chrono_now_utc()` returns an ISO-8601 string in `YYYY-MM-DDTHH:MM:SSZ` format. `trace_ts()` slices it to produce `"YYYY-MM-DD · HH:MM:SS · "` — the `T` separator becomes ` · ` and the trailing `Z` becomes ` · `.

Each production file replaces its `"[trace] "` literal with a `trace_ts()` first argument:

```
Old: eprintln!( "[trace] touch  {}  solo-skip", aq.name )
New: eprintln!( "{}touch  {}  solo-skip", trace_ts(), aq.name )
```

The result line in `fetch.rs` uses a two-argument pattern where the account name is a separate positional:

```
Old: eprintln!( "[trace] {}  result: OK", label )
New: eprintln!( "{}{}  result: OK", trace_ts(), label )
```

This two-argument form is guarded by the BUG-234 MRE structural test in `src/usage/fetch.rs` (see AC-06).

The `#[inline]` attribute satisfies `clippy::missing_inline_in_public_items`. The `#[must_use]` attribute prevents silent discarding of the timestamp string.

### Acceptance Criteria

- **AC-01**: `trace_ts()` is exported as `pub fn` from `claude_profile_core::account`. It is not gated with `#[cfg(test)]`, is not `pub(crate)`, and is unconditionally available to all callers at runtime.
- **AC-02**: `trace_ts()` returns a string matching `"YYYY-MM-DD · HH:MM:SS · "` (UTC). The date portion is `chrono_now_utc()[..10]` and the time portion is `chrono_now_utc()[11..19]`, joined with ` · ` separators.
- **AC-03**: No `"[trace] "` literal string remains in any production `eprintln!` or `writeln!( std::io::stderr(), ...)` call across the 13 affected source files. Every trace line passes `trace_ts()` as the first format argument.
- **AC-04**: `trace_ts()` is unconditional — it does not inspect any trace flag internally. The `if trace { ... }` call-site guard is the gating mechanism; `trace_ts()` is only called inside that guard.
- **AC-05**: All 13 production files use `use claude_profile_core::account::trace_ts` to resolve `trace_ts`. No inline path `claude_profile_core::account::trace_ts()` is used at call sites.
- **AC-06**: The `fetch.rs` structural test (BUG-234 MRE) asserts the two-argument `"{}{}  result: OK"` pattern in the production `eprintln!` line. This is the only trace line where the account name appears as a separate positional argument; the structural test guards against reversion to the single-argument pattern.

### Bugs

_(none)_

### Dependencies

_(none — `trace_ts()` depends only on `chrono_now_utc()` which already exists in `claude_profile_core::account`)_

### CLI Parameters

| File | Relationship |
|------|-------------|
| [cli/param/023_trace.md](../cli/param/023_trace.md) | `trace::` parameter — governs when trace output is emitted; documents the `YYYY-MM-DD · HH:MM:SS · ` prefix format and example trace sessions |

### Features

| File | Relationship |
|------|-------------|
| [009_token_usage.md](009_token_usage.md) | `.usage` — primary command emitting `fetch`, `refresh`, `touch`, `usage`, and subprocess trace lines; trace examples updated |
| [017_token_refresh.md](017_token_refresh.md) | Token refresh subprocess — `refresh` label trace lines in `refresh.rs`; trace format referenced |
| [024_session_touch.md](024_session_touch.md) | Session touch subprocess — `touch` label trace lines in `touch.rs`; trace format referenced |
| [027_account_use_post_switch_touch.md](027_account_use_post_switch_touch.md) | Post-switch touch — `account.use` label trace lines in `account_ops.rs`; trace format referenced |
| [031_account_inspect.md](031_account_inspect.md) | `.account.inspect` — per-endpoint GET trace lines in `account_inspect.rs`; trace format referenced |
| [034_explicit_session_model_override.md](034_explicit_session_model_override.md) | Model override trace lines in `account_ops.rs` |
| [061_solo_token_conservation.md](061_solo_token_conservation.md) | Solo mode — `solo-skip` trace lines in `touch.rs` and `fetch.rs` |
| [063_explicit_ownership_claim.md](063_explicit_ownership_claim.md) | Ownership write trace lines in `accounts.rs` |

### Sources

| File | Relationship |
|------|-------------|
| `claude_profile_core/src/account.rs` | `trace_ts()` implementation; multiple `writeln!( std::io::stderr(), ...)` call sites updated |
| `src/usage/touch.rs` | `touch` label trace lines — solo-skip, not-owned, skip-reason sites |
| `src/usage/fetch.rs` | Account-label trace lines — reading, GET, result-OK/Err, cannot-read-token, skipped sites |
| `src/usage/refresh.rs` | `refresh` label trace lines — solo-skip, should-retry, attempting, refresh-returned-None, token-refreshed, retry-OK/Err sites |
| `src/usage/api.rs` | `usage`/`account.use`/`account.limits` label trace lines — subprocess skipped, model override, and multi-step operation sites |
| `src/commands/account_ops.rs` | `account.use`/`account.limits`/`account.delete` label trace lines |
| `src/commands/accounts.rs` | `accounts` label trace lines — credential-store read, assignee-write, owner-write sites |
| `src/commands/account_inspect.rs` | `account.inspect` label trace lines — per-endpoint GET and result sites |
| `src/commands/account_relogin.rs` | `account.relogin` label — store-path trace line |
| `src/commands/credentials.rs` | Credential-read trace lines |
| `src/commands/account_renewal.rs` | `account.renewal` label — store-path and targets trace lines |
| `src/commands/token_paths.rs` | Path-resolution trace lines |
| `src/commands/limits.rs` | `account.limits` label — store trace line |
| `src/registry.rs` | Help-text strings updated to reference the new timestamp prefix format |

### Tests

| File | Relationship |
|------|-------------|
| `src/usage/api_tests.rs` | Unit-level trace assertions updated to ` · ` sentinel |
| `src/usage/touch_tests.rs` | Touch skip trace assertions updated to ` · touch  ` sentinel |
| `src/usage/fetch.rs` | BUG-234 MRE structural test — asserts `"{}{}  result: OK"` pattern in production `eprintln!` (AC-06) |
| `tests/cli/usage_test.rs` | Integration trace sentinel assertions |
| `tests/cli/usage_feature_test.rs` | Trace line count filter uses ` · ` sentinel |
| `tests/cli/accounts_test.rs` | Per-account read trace assertions |
| `tests/cli/account_mutations_test.rs` | Mutation step trace assertions |
| `tests/cli/account_limits_test.rs` | Limits step trace assertions |
| `tests/cli/account_inspect_test.rs` | Per-endpoint GET trace assertions |
| `tests/cli/credentials_test.rs` | Credential diagnostic trace assertions |
| `tests/cli/set_model_test.rs` | Model override step trace assertions |
| `tests/cli/token_paths_test.rs` | Path resolution trace assertions |
