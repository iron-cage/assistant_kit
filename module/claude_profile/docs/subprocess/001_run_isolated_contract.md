# Subprocess: `run_isolated()` Contract

### Scope

- **Purpose**: Define the API contract, isolation mechanism, and result types for `run_isolated()` — the sole subprocess execution mechanism in `claude_profile`.
- **Responsibility**: Authoritative reference for callers of `run_isolated()`; includes required args, timeout behavior, `IsolatedRunResult` fields, and `RunnerError` variants.
- **In Scope**: Function signature, isolation mechanism, result and error types, timeout behavior, required subprocess args, feature gate, and authorized caller constraint.
- **Out of Scope**: Internal implementation (→ `claude_runner_core/src/isolated.rs`); credential write-back protocol (→ `subprocess/002`); invocation-specific call sites (→ `subprocess/003–005`).

### Purpose

`run_isolated()` spawns the Claude binary in a temporary isolated `HOME` directory containing only the supplied credentials. It is the sole mechanism by which `claude_profile` executes credential-refreshing subprocesses.

### Signature

```rust
fn run_isolated(
    credentials_json : &str,          // OAuth credential JSON to write to temp HOME
    args             : Vec<String>,   // Claude args (e.g. ["--print", "."])
    model            : IsolatedModel, // Model selection; prepended as --model <id> unless KeepCurrent
    timeout_secs     : u64,           // 0 = unlimited; 35 for credential refresh pings
) -> Result<IsolatedRunResult, RunnerError>
```

### Isolation Mechanism

1. Create temp dir `/tmp/isolated-*`
2. Write `credentials_json` to `<temp>/.claude/.credentials.json`
3. Write `ISOLATED_CLAUDE_MD` to `<temp>/.claude/CLAUDE.md` (suppress extended thinking, tool use, questions)
4. Spawn `claude` with `HOME=<temp>`, piped stdout/stderr
5. Monitor subprocess up to `timeout_secs`; kill on timeout but still harvest output
6. Compare `<temp>/.claude/.credentials.json` with original after completion
7. Remove temp dir unconditionally

### `IsolatedRunResult`

| Field | Type | Semantics |
|-------|------|-----------|
| `exit_code` | `i32` | Process exit code; `-1` if terminated without exit code (timeout kill) |
| `stdout` | `String` | Captured standard output |
| `stderr` | `String` | Captured standard error |
| `credentials` | `Option<String>` | Updated credentials JSON if the file changed; `None` if byte-identical or unreadable |

### `RunnerError` Variants

| Variant | When |
|---------|------|
| `ClaudeNotFound` | `claude` binary absent from `PATH` |
| `TempDirFailed(String)` | Cannot create temp directory |
| `Timeout { secs }` | Subprocess exceeded `timeout_secs` (no output captured) |
| `TimeoutWithOutput { secs, partial_stdout }` | Subprocess timed out; partial stdout was captured (Fix BUG-243) |
| `Io(String)` | File write, read, or cleanup failure |

### Timeout Behavior

- `timeout_secs = 0` → **unlimited** (no deadline). Used for relogin subprocesses. Fix(I2) in `claude_runner_core/src/isolated.rs`.
- `timeout_secs = 35` → standard credential refresh timeout. When the credential file changes before timeout fires, `run_isolated` returns `Ok(IsolatedRunResult { credentials: Some(new_json), exit_code: -1 })` — updated credentials are captured even if the subprocess is killed by timeout.

### Required Subprocess Args

`["--print", "."]` — the ONLY correct invocation for credential refresh pings.

**Broken alternatives:**
- `[]` (no args): Claude Code in non-TTY mode with no args detects nothing to do, exits without OAuth refresh. `credentials = None` always. (BUG-169)
- `["--print", ".", "--max-tokens", "1"]`: `--max-tokens 1` triggers API rejection before OAuth exchange. (TSK-151)

### Feature Gate

`run_isolated()` is compiled only under `#[cfg(feature = "enabled")]`. Callers in `claude_profile` must handle the offline build case — the parameter is accepted but no subprocess is spawned.

### Sole Authorized Caller

All token refresh operations MUST go through `account::refresh_account_token()` in `claude_profile_core`. Direct `run_isolated()` calls for credential refresh are forbidden. See [invariant/008](../invariant/008_single_token_refresh_entry.md).

### Sources

| File | Relationship |
|------|-------------|
| `claude_runner_core/src/isolated.rs` | Implementation source |

### Subprocess

| File | Relationship |
|------|-------------|
| [subprocess/002](002_credential_writeback.md) | How `credentials` field flows back to disk |
| [subprocess/003](003_token_refresh_invocation.md) | Token refresh invocation |
| [subprocess/004](004_session_touch_invocation.md) | Session touch invocation |

### Invariants

| File | Relationship |
|------|-------------|
| [invariant/008](../invariant/008_single_token_refresh_entry.md) | Single-entry-point invariant |
