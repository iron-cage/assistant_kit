# Feature: Isolated Subprocess Execution

### Scope

- **Purpose**: Provide a way to spawn `claude` with a temporary, isolated HOME directory containing only a single credential file, capture the exit code and output, detect any credential changes written by the subprocess, and return them to the caller — all without contaminating the host environment.
- **Responsibility**: Documents the `run_isolated()` function API, the `IsolatedRunResult` and `RunnerError` types, the temp-HOME construction algorithm, the thread-based timeout mechanism, credential change detection by byte comparison, unconditional temp-dir cleanup, and the single-execution-point constraint.
- **In Scope**: `run_isolated()` signature and contract; `IsolatedRunResult` and `RunnerError` types; temp HOME construction; SIGINT/timeout handling via `mpsc::channel + recv_timeout`; credential write-back detection; thread ownership and cleanup; `#[cfg(feature = "enabled")]` gate; `lim_it` test categorisation.
- **Out of Scope**: `refresh::` retry logic (→ `claude_profile/docs/feature/017_token_refresh.md`); how callers use the returned credentials; `run_isolated()` argument construction (caller responsibility).

### Design

`run_isolated()` spawns `claude` with `HOME` overridden to a temporary directory containing only a single `.claude.json` credentials file. When `claude` completes (or times out), the function checks whether the subprocess wrote updated credentials back to that temp HOME. The result — exit code, captured stdout/stderr, and optionally refreshed credentials JSON — is returned to the caller without modifying any host-environment files.

**Types (always available — no `#[cfg(feature = "enabled")]` gate on types):**

```rust
pub struct IsolatedRunResult {
    pub exit_code: i32,
    pub stdout:    String,
    pub stderr:    String,
    pub credentials: Option<String>,   // Some(new_json) when claude rewrote credentials
}

pub enum RunnerError {
    ClaudeNotFound,
    TempDirFailed(String),
    Timeout { secs: u64 },
    Io(String),
}
```

`IsolatedRunResult` and `RunnerError` are defined in `src/isolated.rs` and re-exported from `src/lib.rs`. They are unconditionally available so callers can name the types in function signatures and test code without `#[cfg]` guards.

**Function signature:**

```rust
#[cfg(feature = "enabled")]
pub fn run_isolated(
    credentials_json: &str,
    args:             Vec<String>,
    timeout_secs:     u64,
) -> Result<IsolatedRunResult, RunnerError>
```

**Algorithm:**

```
1.  create temp dir: {tmp_dir}/claude_isolated_{pid}
    on failure → RunnerError::TempDirFailed
2.  write credentials_json to <temp>/.claude/.credentials.json
    on write failure → cleanup temp, return RunnerError::Io
3.  build command via ClaudeCommand::new().with_home(<temp>).with_args(args):
      claude <args...>
      env HOME=<temp>
      (all other env vars inherited from parent process)
      stdout and stderr piped
4.  spawn command (single execution point: ClaudeCommand::execute())
    if "claude" binary not found → RunnerError::ClaudeNotFound
5.  transfer Child ownership to thread T
    T calls execute() → sends result via mpsc::Sender
6.  read credentials from <temp>/.claude/.credentials.json (before cleanup — order matters)
    compare bytes with original credentials_json argument
    if different AND valid UTF-8: credentials = Some(new_json)
    else: credentials = None
7.  unconditionally remove temp dir (even on timeout or error paths)
8.  caller blocks on receiver.recv_timeout(timeout_secs)
    on timeout AND credentials is Some: return Ok (credentials refreshed before blocking)
    on timeout AND credentials is None: RunnerError::Timeout { secs: timeout_secs }
    on Ok(output): return Ok(IsolatedRunResult { exit_code, stdout, stderr, credentials })
```

**Timeout-with-credentials fix (`issue-isolated-credentials-on-timeout`):** Claude Code refreshes OAuth tokens at startup before blocking for user input. A subprocess that successfully refreshes credentials may then block waiting for a message — triggering the timeout before producing any output. In this case, the subprocess has already written refreshed credentials to `<temp>/.claude/.credentials.json`. The fix: credentials are read from disk _before_ the timeout check. If timeout fires but `credentials` is `Some`, `run_isolated()` returns `Ok` so callers receive the refreshed credentials despite the timeout.

**Single-execution-point constraint:**

`claude_runner_core` contains exactly one `Command::new("claude")` call site (invariant: `docs/invariant/001_single_execution_point.md`). `run_isolated()` must satisfy this invariant by adding a `with_home(path: &Path)` builder method to `ClaudeCommand` and routing execution through the existing `execute()` infrastructure rather than adding a bare `Command::new("claude")` in `src/isolated.rs`.

**Timeout mechanism:**

`std::process::Child::wait_with_output()` blocks indefinitely. To implement a timeout without a `tokio` dependency, the child is sent to a dedicated OS thread via ownership transfer. The calling thread waits on `mpsc::channel::Receiver::recv_timeout(duration)`. On timeout, the receiver drops and the caller kills the child process by PID (`child.kill()` is called on the `Child` value stored in the thread before the thread is abandoned).

**Temp HOME isolation:**

The temp directory structure is:

```
{tmp}/claude_isolated_{pid}/
  .claude/
    .credentials.json     ← credentials_json written here
```

`HOME` is set to `{tmp}/claude_isolated_{pid}`. Other env vars are inherited. The subprocess sees a fresh `~/.claude/.credentials.json` with the provided credentials and nothing else — no other `~/.claude/` files, no `settings.json`, no session state.

**Credential change detection:**

After the subprocess exits (or before cleanup on timeout), the function reads `.claude/.credentials.json` back from the temp HOME and compares it byte-by-byte with the original `credentials_json` input. If the bytes differ and the content is valid UTF-8, `credentials: Some(new_json)` is returned. If the subprocess did not modify the file, or the file is unreadable, `credentials: None` is returned. The comparison is exact — no JSON normalisation.

**Caller note — `expiresAt` is NOT updated by the subprocess:** Claude Code's OAuth refresh exchange updates `accessToken` and `refreshToken` in the credentials file but does NOT write `expiresAt`. Callers that need the post-refresh token expiry must derive it from the JWT `exp` claim of the returned `accessToken` (base64url-decode the second `.`-separated segment, parse `"exp"` as Unix seconds, multiply by 1000 for ms) rather than reading `expiresAt` from the credentials file. See BUG-162.

**Unconditional cleanup:**

The temp directory is removed in all code paths: success, timeout, and I/O error. A failed cleanup is logged (best-effort) but does not cause `run_isolated()` to return an error.

**Feature gate:**

`run_isolated()` is compiled only under `#[cfg(feature = "enabled")]`. When `enabled` is absent the function does not exist; callers must gate their calls with the same attribute. `IsolatedRunResult` and `RunnerError` are always compiled (no feature gate) so test code and type-only references always compile.

### Acceptance Criteria

- **AC-33**: `run_isolated(creds_json, args, timeout_secs)` spawns `claude` with `HOME` overridden to a temp directory containing only `.claude/.credentials.json` populated with `creds_json`.
- **AC-34**: When the subprocess exits normally, `run_isolated()` returns `Ok(IsolatedRunResult)` with the correct `exit_code`, `stdout`, and `stderr`.
- **AC-35**: When the subprocess rewrites `.claude.json`, `credentials` is `Some(new_json)` with the updated content.
- **AC-36**: When the subprocess does not modify `.claude.json`, `credentials` is `None`.
- **AC-37**: When `timeout_secs` elapses before the subprocess exits, `run_isolated()` returns `Err(RunnerError::Timeout { secs })` and terminates the child process.
- **AC-38**: The temp directory is removed in all code paths — success, timeout, and I/O error — with no temp-dir leak.
- **AC-39**: `run_isolated()` does not call `Command::new("claude")` directly; it routes through `ClaudeCommand::with_home()` and the existing `execute()` path (single-execution-point invariant).
- **AC-40**: `IsolatedRunResult` and `RunnerError` are available without `#[cfg(feature = "enabled")]`; `run_isolated()` is available only with it.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/isolated.rs` | `run_isolated()` implementation; `IsolatedRunResult`, `RunnerError` types |
| source | `src/lib.rs` | Re-exports `IsolatedRunResult`, `RunnerError`, `run_isolated` |
| source | `src/command.rs` | `ClaudeCommand::with_home()` builder method (single execution point) |
| invariant | [invariant/001_single_execution_point.md](../invariant/001_single_execution_point.md) | `Command::new("claude")` must appear exactly once |
| task | `task/claude_runner_core/136_run_isolated_subprocess.md` | Implementation task for this feature |
| dep | `claude_profile` | `usage.rs` — `refresh::` retry logic calling `run_isolated()` |
| doc | `claude_profile/docs/feature/017_token_refresh.md` | Caller for token refresh: `refresh::` param, credential write-back |
