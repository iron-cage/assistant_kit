# Feature: Isolated Subprocess Execution

### Scope

- **Purpose**: Provide a way to spawn `claude` with a temporary, isolated HOME directory containing only a single credential file, capture the exit code and output, detect any credential changes written by the subprocess, and return them to the caller — all without contaminating the host environment.
- **Responsibility**: Documents the `run_isolated()` function API, the `IsolatedRunResult`, `RunnerError`, and `IsolatedModel` types, the temp-HOME construction algorithm, the `spawn_piped()` + `try_wait()` polling timeout mechanism, CLAUDE.md write to isolated HOME, chrome flag suppression, credential change detection by byte comparison, unconditional temp-dir cleanup, and the single-execution-point constraint.
- **In Scope**: `run_isolated()` signature and contract; `IsolatedRunResult`, `RunnerError` (including `TimeoutWithOutput` variant), and `IsolatedModel` types; temp HOME construction; timeout handling via `spawn_piped()` + `try_wait()` polling with 50 ms ticks; CLAUDE.md write to `<temp>/.claude/CLAUDE.md`; `--chrome` suppression in isolated mode; credential write-back detection; `#[cfg(feature = "enabled")]` gate; `lim_it` test categorisation.
- **Out of Scope**: `refresh::` retry logic (→ `claude_profile/docs/feature/017_token_refresh.md`); how callers use the returned credentials; `run_isolated()` argument construction (caller responsibility).

### Design

`run_isolated()` spawns `claude` with `HOME` overridden to a temporary directory containing `.claude/.credentials.json` (credentials) and `.claude/CLAUDE.md` (instructions to respond immediately without extended thinking). When `claude` completes (or times out), the function checks whether the subprocess wrote updated credentials back to that temp HOME. The result — exit code, captured stdout/stderr, and optionally refreshed credentials JSON — is returned to the caller without modifying any host-environment files.

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
    /// Timeout with no partial output captured.
    Timeout { secs: u64 },
    /// Timeout with partial stdout captured (Fix BUG-243: old Timeout discarded all buffered output).
    TimeoutWithOutput { secs: u64, partial_stdout: String },
    Io(String),
}

/// Default model ID injected by IsolatedModel::Default.
pub const ISOLATED_DEFAULT_MODEL: &str = "claude-opus-4-6";

pub enum IsolatedModel {
    Default,           // prepends --model claude-opus-4-6
    KeepCurrent,       // no --model flag; Claude binary chooses
    Specific(String),  // prepends --model <id>
}

impl IsolatedModel {
    /// Returns the model ID string to inject via `--model`, or `None` for `KeepCurrent`.
    /// Used internally by `run_isolated()` (algorithm step 3) to build the arg list.
    pub fn model_id(&self) -> Option<&str>;
}
```

`IsolatedRunResult`, `RunnerError`, `IsolatedModel`, and `ISOLATED_DEFAULT_MODEL` are defined in `src/isolated.rs` and re-exported from `src/lib.rs`. They are unconditionally available so callers can name the types in function signatures and test code without `#[cfg]` guards.

**Function signature:**

```rust
#[cfg(feature = "enabled")]
pub fn run_isolated(
    credentials_json: &str,
    args:             Vec<String>,
    timeout_secs:     u64,
    model:            IsolatedModel,
) -> Result<IsolatedRunResult, RunnerError>
```

**Algorithm:**

```
1.  create temp dir: {tmp_dir}/claude_isolated_{pid}
    on failure → RunnerError::TempDirFailed
2.  write credentials_json to <temp>/.claude/.credentials.json
    on write failure → cleanup temp, return RunnerError::Io
2b. write minimal CLAUDE.md to <temp>/.claude/CLAUDE.md
    content instructs subprocess to respond immediately without extended thinking
    on write failure → cleanup temp, return RunnerError::Io
3.  build command; if model != KeepCurrent, prepend ["--model", <id>] to args:
      ClaudeCommand::new().with_home(<temp>).with_args([--model <id>, <args...>])
      env HOME=<temp>
      (all other env vars inherited from parent process)
      stdout and stderr piped
      NOTE: isolated subprocesses keep chrome active — user tasks may invoke browser tools
4.  spawn command via spawn_piped() (single execution point: ClaudeCommand)
    if "claude" binary not found → RunnerError::ClaudeNotFound
5.  poll subprocess in 50ms ticks until exit or deadline:
      try_wait() → Some(_): subprocess exited; break to step 6
      try_wait() → None AND now >= deadline: timeout; kill child; break to step 5b
      try_wait() → Err: return RunnerError::Io
5b. (timeout path only) kill() + wait_with_output() to collect partial stdout
6.  read credentials from <temp>/.claude/.credentials.json (before cleanup — order matters)
    compare bytes with original credentials_json argument
    if different AND valid UTF-8: credentials = Some(new_json)
    else: credentials = None
7.  unconditionally remove temp dir (even on timeout or error paths)
8.  if timed_out AND credentials is Some: return Ok (credentials refreshed before timeout fired)
    if timed_out AND credentials is None: return RunnerError::TimeoutWithOutput { partial_stdout }
    else: return Ok(IsolatedRunResult { exit_code, stdout, stderr, credentials })
```

**Timeout-with-credentials fix (`issue-isolated-credentials-on-timeout`):** Claude Code refreshes OAuth tokens at startup before blocking for user input. A subprocess that successfully refreshes credentials may then block waiting for a message — triggering the timeout before producing any output. In this case, the subprocess has already written refreshed credentials to `<temp>/.claude/.credentials.json`. The fix: credentials are read from disk _before_ the timeout check. If timeout fires but `credentials` is `Some`, `run_isolated()` returns `Ok` so callers receive the refreshed credentials despite the timeout.

**Single-execution-point constraint:**

`claude_runner_core` contains exactly one `Command::new("claude")` call site (invariant: `docs/invariant/001_single_execution_point.md`). `run_isolated()` must satisfy this invariant by adding a `with_home(path: &Path)` builder method to `ClaudeCommand` and routing execution through the existing `execute()` infrastructure rather than adding a bare `Command::new("claude")` in `src/isolated.rs`.

**Timeout mechanism:**

`run_isolated()` uses `spawn_piped()` + `try_wait()` polling in 50 ms ticks. The `Child` handle stays in scope through the timeout deadline, enabling `child.kill()` + `child.wait_with_output()` to recover any buffered output on timeout (Fix BUG-243: the previous thread/channel approach abandoned the `Child` inside a spawned thread on `recv_timeout`, making buffered stdout irrecoverable). On timeout, `RunnerError::TimeoutWithOutput { partial_stdout }` is returned with any output that was buffered before the kill.

**Temp HOME isolation:**

The temp directory structure is:

```
{tmp}/claude_isolated_{pid}/
  .claude/
    .credentials.json     ← credentials_json written here
    CLAUDE.md             ← minimal instruction file: respond immediately, no extended thinking
```

`HOME` is set to `{tmp}/claude_isolated_{pid}`. Other env vars are inherited. The subprocess sees a fresh `~/.claude/.credentials.json` with the provided credentials and a `CLAUDE.md` that instructs it to respond immediately without extended thinking — preventing timeout on keep-alive `--print .` prompts. No other `~/.claude/` files are present (no `settings.json`, no session state). Isolated subprocesses keep the default chrome setting active — user tasks may invoke browser tools. Chrome suppression (`--no-chrome`) applies only to credential-refresh subprocesses (`clr refresh`), where browser access is never needed.

**Credential change detection:**

After the subprocess exits (or before cleanup on timeout), the function reads `.claude/.credentials.json` back from the temp HOME and compares it byte-by-byte with the original `credentials_json` input. If the bytes differ and the content is valid UTF-8, `credentials: Some(new_json)` is returned. If the subprocess did not modify the file, or the file is unreadable, `credentials: None` is returned. The comparison is exact — no JSON normalisation.

**Caller note — `expiresAt` is NOT updated by the subprocess:** Claude Code's OAuth refresh exchange updates `accessToken` and `refreshToken` in the credentials file but does NOT write `expiresAt`. Callers that need the post-refresh token expiry must derive it from the JWT `exp` claim of the returned `accessToken` (base64url-decode the second `.`-separated segment, parse `"exp"` as Unix seconds, multiply by 1000 for ms) rather than reading `expiresAt` from the credentials file. See BUG-162.

**Unconditional cleanup:**

The temp directory is removed in all code paths: success, timeout, and I/O error. A failed cleanup is logged (best-effort) but does not cause `run_isolated()` to return an error.

**Feature gate:**

`run_isolated()` is compiled only under `#[cfg(feature = "enabled")]`. When `enabled` is absent the function does not exist; callers must gate their calls with the same attribute. `IsolatedRunResult` and `RunnerError` are always compiled (no feature gate) so test code and type-only references always compile.

### Acceptance Criteria

- **AC-33**: `run_isolated(creds_json, args, timeout_secs, model)` spawns `claude` with `HOME` overridden to a temp directory containing `.claude/.credentials.json` (populated with `creds_json`) and `.claude/CLAUDE.md` (minimal instruction file directing Claude to respond immediately without extended thinking). When `model` is not `KeepCurrent`, `--model <id>` is prepended to `args` before the subprocess is spawned.
- **AC-41**: `with_home_isolation()` is a `ClaudeCommand` builder method that suppresses `--chrome` by calling `with_chrome(None)`. It is used by credential-refresh subprocesses (`clr refresh`), not by `run_isolated()` — OAuth exchange is a pure HTTP operation requiring no browser access. Isolated subprocesses keep chrome active; user tasks may invoke browser tools.
- **AC-42**: The `CLAUDE.md` written to `<temp>/.claude/CLAUDE.md` contains at minimum the instruction: respond immediately to `--print` prompts without extended thinking, no preamble, no tool use. This prevents isolated subprocesses from entering extended thinking mode on keep-alive prompts, which would cause them to exceed the subprocess timeout.
- **AC-34**: When the subprocess exits normally, `run_isolated()` returns `Ok(IsolatedRunResult)` with the correct `exit_code`, `stdout`, and `stderr`.
- **AC-35**: When the subprocess rewrites `.claude.json`, `credentials` is `Some(new_json)` with the updated content.
- **AC-36**: When the subprocess does not modify `.claude.json`, `credentials` is `None`.
- **AC-37**: When `timeout_secs` elapses before the subprocess exits and no credentials were refreshed, `run_isolated()` kills the child process and returns `Err(RunnerError::TimeoutWithOutput { secs, partial_stdout })` where `partial_stdout` contains any output captured before the kill. When credentials were refreshed before the timeout fired, returns `Ok` so callers receive the refreshed credentials.
- **AC-38**: The temp directory is removed in all code paths — success, timeout, and I/O error — with no temp-dir leak.
- **AC-39**: `run_isolated()` does not call `Command::new("claude")` directly; it routes through `ClaudeCommand::with_home()` and the existing `execute()` path (single-execution-point invariant).
- **AC-40**: `IsolatedRunResult`, `RunnerError`, `IsolatedModel`, and `ISOLATED_DEFAULT_MODEL` are available without `#[cfg(feature = "enabled")]`; `run_isolated()` is available only with it.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/isolated.rs` | `run_isolated()` implementation; `IsolatedRunResult`, `RunnerError` types; `ISOLATED_CLAUDE_MD` constant |
| source | `src/lib.rs` | Re-exports `IsolatedRunResult`, `RunnerError`, `IsolatedModel`, `ISOLATED_DEFAULT_MODEL`, `run_isolated` |
| source | `src/command/mod.rs` | `ClaudeCommand` builder; `with_home()` method; chrome injection logic |
| source | `src/command/params_core.rs` | `with_home_isolation()` method — chains `with_chrome(None)` to suppress chrome in refresh mode |
| invariant | [invariant/001_single_execution_point.md](../invariant/001_single_execution_point.md) | `Command::new("claude")` must appear exactly once |
| task | `task/claude_runner_core/136_run_isolated_subprocess.md` | Implementation task for this feature |
| dep | `claude_profile` | `usage.rs` — `refresh::` retry logic calling `run_isolated()` |
| doc | `claude_profile/docs/feature/017_token_refresh.md` | Caller for token refresh: `refresh::` param, credential write-back |
