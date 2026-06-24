# CLI Type: ErrorClass

Caller-facing taxonomy that groups the 12 observable CLI error conditions into 8 semantic classes.
Use this taxonomy to decide the appropriate caller response at the application layer.

- **Purpose:** Semantic grouping of CLI error conditions for caller response decisions
- **Fundamental Type:** taxonomy (8 classes, not an enum — used for documentation and caller guidance)
- **Constants:** see below
- **Constraints:** documentation type only; runtime classification uses `ErrorKind` (type 13) for subprocess errors
- **Parsing:** not a CLI input or runtime type — reference taxonomy for documentation
- **Methods:** —

### Error Class Table

All retriable classes use a uniform parameter pair: `--retry-on-<class>` (u8, count) and `--<class>-delay` (u32, seconds).
All classes default to **retry = 2**, **delay = 30s** (Validation delay = 0s — no server-side throttle).

| Error Class | Detection | Retry Param | Delay Param | Default Retry | Default Delay | Caller Action |
|-------------|-----------|-------------|-------------|---------------|---------------|---------------|
| **Transient** | exit 2, no quota text | `--retry-on-transient` | `--transient-delay` | 2 | 30s | Retry with backoff |
| **Account** | `"You've hit your limit"` in output | `--retry-on-account` | `--account-delay` | 2 | 30s | Retry with backoff; or switch credentials |
| **Auth** | `"authentication_error"` or `"Your organization does not have access to Claude"` in output | `--retry-on-auth` | `--auth-delay` | 2 | 30s | Retry with backoff; or fix credentials |
| **Service** | `"API Error: "` in output | `--retry-on-service` | `--service-delay` | 2 | 30s | Retry with backoff |
| **Process** | exit > 128 (signal) or exit 4 (timeout) | `--retry-on-process` | `--process-delay` | 2 | 30s | Retry with backoff; investigate persistent failures |
| **Validation** | exit 3 (`--expect` mismatch) | `--retry-on-validation` | `--validation-delay` | 2 | 0s | Re-prompt immediately; adjust `--expect` on persistent mismatch |
| **Runner** | exit 1 before subprocess | `--retry-on-runner` | `--runner-delay` | 2 | 30s | Retry with backoff; fix environment on persistent failure |
| **Unknown** | any other non-zero exit | `--retry-on-unknown` | `--unknown-delay` | 2 | 30s | Retry with backoff; surface and investigate |

### Class Descriptions

**Transient** — Temporary rate-limiting by the Anthropic API (HTTP 429). The subprocess exits 2 with no distinguishing text. Automatic retry via `--retry-on-transient N` with `--transient-delay SECS` cooldown is the correct response.

**Account** — Period quota exhausted for the current account. Distinguished from `Transient` by the `"You've hit your limit"` text in output; also exits 2. Retries with the same 3-tier resolution as all other classes (effective default = 2 retries, 30s delay). For immediate account switching instead of retrying, see the deferred `--on-quota-exhausted` parameter.

**Service** — API-layer error from the Anthropic backend (HTTP 4xx/5xx). The `"API Error: "` prefix (colon-space, not parenthesis) identifies these. May be transient infrastructure issues. Automatic retry via `--retry-on-service N` with `--service-delay SECS` cooldown.

**Auth** — Credential or authorization failure. The subprocess rejects the current credentials. Detected by `"authentication_error"` in output (Fix BUG-314: this pattern fires before the `"API Error: "` catch-all, covering the Claude CLI 401 form) or by `"Your organization does not have access to Claude"`. Rotating or re-issuing credentials is required. When auth error is detected and no credential recovery hook (`--on-auth-error switch`) is configured, the retry loop exits immediately (fail-fast) without sleeping or consuming retry slots (Fix BUG-315).

**Process** — Subprocess died from an OS signal or was killed by the CLR timeout watchdog. `Signal` variants have exit code > 128; `Timeout` variants have exit 4 with the `"Error: timeout after {N}s"` stderr line. Increasing `--timeout` or investigating external process killers is the response.

**Validation** — Output did not match the `--expect` pattern within the allowed `--retry-on-validation` count. The CLR layer exits 3. Adjust the pattern or increase retry count.

**Runner** — CLR infrastructure failure before or after subprocess execution: binary not found in PATH, OS spawn error, session gate timed out waiting for a slot, or output file write failure. All exit 1. Fix the environment rather than retrying.

**Unknown** — Non-zero exit with no recognized pattern, exit code not 2, exit code ≤ 128. Automatic retry via `--retry-on-unknown N` with `--unknown-delay SECS` cooldown. Surface raw stdout/stderr; investigate subprocess logs.

### Exit Code to Class Mapping

| Exit Code | Possible Classes | Disambiguation |
|-----------|-----------------|----------------|
| 0 | — | Success (no class) |
| 1 | Runner | BinaryNotFound / SpawnFailed / GateTimeout / OutputFileError |
| 2 | Transient, Account | Text contains `"hit your limit"` → Account; otherwise → Transient |
| 3 | Validation | ExpectMismatch (CLR layer) |
| 4 | Process | Timeout (CLR watchdog); stderr contains `"Error: timeout after {N}s"` |
| > 128 | Process | Signal (POSIX 128+N convention) |
| other | Unknown, Auth, Service | Check stdout/stderr text for `"API Error: "` or `"Your organization"` |

### Strategy Configuration — 3-Tier Parameter Hierarchy

Resolution logic per class per invocation:

```
effective_count(class)  = --retry-override        ?? --retry-on-<class>  ?? --retry-default (2)
effective_delay(class)  = --retry-override-delay  ?? --<class>-delay     ?? --retry-default-delay (30s)
```

(`??` = use left operand if explicitly set; else fall through to right)

#### Tier 1 — Override

Beats class-specific values when set; default is unset (auto).

| Param                    | Env Var                    | Type | Default | Effect                        |
|--------------------------|----------------------------|------|---------|-------------------------------|
| `--retry-override`       | `CLR_RETRY_OVERRIDE`       | u8   | auto    | Forces count for all classes  |
| `--retry-override-delay` | `CLR_RETRY_OVERRIDE_DELAY` | u32  | auto    | Forces delay for all classes  |

#### Tier 2 — Class-Specific

Default is `auto` — inherits from Tier 3 fallback when not explicitly set.

| Error Class    | Count Param              | Delay Param              | Default | Env Vars (count / delay)                                      |
|----------------|--------------------------|--------------------------|---------|---------------------------------------------------------------|
| **Transient**  | `--retry-on-transient`   | `--transient-delay`      | auto    | `CLR_RETRY_ON_TRANSIENT` / `CLR_TRANSIENT_DELAY`              |
| **Account**    | `--retry-on-account`     | `--account-delay`        | auto    | `CLR_RETRY_ON_ACCOUNT` / `CLR_ACCOUNT_DELAY`                  |
| **Auth**       | `--retry-on-auth`        | `--auth-delay`           | auto    | `CLR_RETRY_ON_AUTH` / `CLR_AUTH_DELAY`                        |
| **Service**    | `--retry-on-service`     | `--service-delay`        | auto    | `CLR_RETRY_ON_SERVICE` / `CLR_SERVICE_DELAY`                  |
| **Process**    | `--retry-on-process`     | `--process-delay`        | auto    | `CLR_RETRY_ON_PROCESS` / `CLR_PROCESS_DELAY`                  |
| **Validation** | `--retry-on-validation`  | `--validation-delay`     | auto    | `CLR_RETRY_ON_VALIDATION` / `CLR_VALIDATION_DELAY`            |
| **Runner**     | `--retry-on-runner`      | `--runner-delay`         | auto    | `CLR_RETRY_ON_RUNNER` / `CLR_RUNNER_DELAY`                    |
| **Unknown**    | `--retry-on-unknown`     | `--unknown-delay`        | auto    | `CLR_RETRY_ON_UNKNOWN` / `CLR_UNKNOWN_DELAY`                  |

#### Tier 3 — Fallback

Concrete defaults applied whenever the class-specific value is `auto`.

| Param                   | Env Var                    | Type | Default | Effect                       |
|-------------------------|----------------------------|------|---------|------------------------------|
| `--retry-default`       | `CLR_RETRY_DEFAULT`        | u8   | 2       | Count for all unset classes  |
| `--retry-default-delay` | `CLR_RETRY_DEFAULT_DELAY`  | u32  | 30s     | Delay for all unset classes  |

**Total retry parameters: 20** (2 override + 16 class-specific + 2 fallback)

**Examples:**

```sh
# All classes: 5 retries, 60s delay — one flag pair
clr run "task" --retry-override 5 --retry-override-delay 60

# Set fallback to 3 retries; disable Transient retry specifically
clr run "task" --retry-default 3 --retry-on-transient 0

# Tune only Service class; everything else uses fallback (2 retries / 30s)
clr run "task" --retry-on-service 5 --service-delay 10
```

**Required param renames (6):**

| Old Param | New Param | Old Env Var | New Env Var |
|-----------|-----------|-------------|-------------|
| `--retry-on-rate-limit` | `--retry-on-transient` | `CLR_RETRY_ON_RATE_LIMIT` | `CLR_RETRY_ON_TRANSIENT` |
| `--retry-delay` | `--transient-delay` | `CLR_RETRY_DELAY` | `CLR_TRANSIENT_DELAY` |
| `--retry-on-api-error` | `--retry-on-service` | `CLR_RETRY_ON_API_ERROR` | `CLR_RETRY_ON_SERVICE` |
| `--api-error-delay` | `--service-delay` | `CLR_API_ERROR_DELAY` | `CLR_SERVICE_DELAY` |
| `--retry-on-unknown-error` | `--retry-on-unknown` | `CLR_RETRY_ON_UNKNOWN_ERROR` | `CLR_RETRY_ON_UNKNOWN` |
| `--expect-retries` | `--retry-on-validation` | `CLR_EXPECT_RETRIES` | `CLR_RETRY_ON_VALIDATION` |

**New params (14):** `--retry-on-account`, `--account-delay`, `--retry-on-auth`, `--auth-delay`, `--retry-on-process`, `--process-delay`, `--validation-delay`, `--retry-on-runner`, `--runner-delay`, `--unknown-delay`, `--retry-override`, `--retry-override-delay`, `--retry-default`, `--retry-default-delay`

**Deferred parameters** (2 total — require multi-credential infrastructure):

| CLI Parameter | Env Var | Type | Default | Error Class | Prerequisite |
|--------------|---------|------|---------|-------------|--------------|
| `--on-quota-exhausted` | `CLR_ON_QUOTA_EXHAUSTED` | enum (abort\|switch) | abort | Account | `--creds` list or directory |
| `--on-auth-error` | `CLR_ON_AUTH_ERROR` | enum (abort\|switch) | abort | Auth | `--creds` list or directory |

**Note:** The `switch` option for Account and Auth classes requires multi-credential infrastructure — a `--creds` list or directory from which `clr` can select an alternate credential file after an account exhaustion or auth failure. This is a prerequisite dependency, not a simple parameter addition.

### Configuration Tiers

Current system uses 3 tiers. Target is 4 tiers with a config file layer.

| Priority | Tier | Source | Status |
|----------|------|--------|--------|
| 1 (highest) | CLI parameter | `--flag value` on command line | Exists (53 params) |
| 2 | Environment variable | `CLR_*` env vars | Exists (56 vars) |
| 3 | Config file | TBD — `~/.config/clr/config.toml` or `$CLR_CONFIG` | Gap |
| 4 (lowest) | Hardcoded default | Built into source code | Exists |

**Config file gap:** No config file mechanism exists. Config keys would use `snake_case` matching CLI `--kebab-case` names (e.g., `retry_on_transient = 1`). All 53 parameters should be configurable at the config file tier.

### Console Output Format

The clr error line **embeds the original subprocess message**, augmented by the error class tag.

Two line patterns — always written to stderr:

- **Retry progress:** `[Class] <original-message> — retrying in Xs (attempt M/N)…`
- **Terminal error:** `Error: [Class] <original-message> (exit N)`

Rules:
- `[Class]` tag is always present — machine consumers match on it
- `<original-message>` is the first meaningful line from subprocess stdout or stderr
- For classes with no subprocess text (Signal, Timeout, Runner), clr supplies the description
- `— retries exhausted` suffix replaces `— retrying in Xs (attempt M/N)…` on final failure
- The raw full subprocess output may precede the structured error line when it is multi-line

**Transient / RateLimit** (exit 2, subprocess emits rate-limit text):
```
[Transient] You are being rate limited. Please wait before retrying. — retrying in 30s (attempt 1/3)…
[Transient] You are being rate limited. Please wait before retrying. — retrying in 30s (attempt 2/3)…
Error: [Transient] You are being rate limited. Please wait before retrying. — retries exhausted (exit 2)
```
Without retry configured:
```
Error: [Transient] You are being rate limited. Please wait before retrying. (exit 2)
```
When subprocess emits no text (silent exit 2):
```
Error: [Transient] rate limit (exit 2)
```

**Account / QuotaExhausted** (exit 2, text distinguishes from Transient):
```
Error: [Account] You've hit your limit. Please try again after your billing period resets. (exit 2)
```

**Service / ApiError** (exit 1, `"API Error: "` prefix in output):
```
[Service] API Error: 503 Service Unavailable — retrying in 30s (attempt 1/2)…
Error: [Service] API Error: 503 Service Unavailable — retries exhausted (exit 1)
```
Without retry configured:
```
Error: [Service] API Error: 503 Service Unavailable (exit 1)
```

**Auth / AuthError** (exit 1):
```
Error: [Auth] Your organization does not have access to Claude. Please check your subscription. (exit 1)
```

**Process / Signal** (exit 143 — no subprocess text, killed by OS signal):
```
Error: [Process] terminated by signal (exit 143)
```

**Process / Timeout** (exit 4 — CLR watchdog killed the subprocess):
```
Error: [Process] timeout after 30s (exit 4)
```

**Validation / ExpectMismatch** (exit 3 — `--expect` pattern not matched):
```
[Validation] expected "yes|no", got "maybe" — retrying (attempt 1/2)…
Error: [Validation] expected "yes|no", got "perhaps" — retries exhausted (exit 3)
```
Without retry configured:
```
Error: [Validation] expected "yes|no", got "maybe" (exit 3)
```

**Runner / BinaryNotFound** (exit 1 — before spawn, no subprocess):
```
Error: [Runner] claude binary not found in PATH — install with: npm i -g @anthropic-ai/claude-code (exit 1)
```

**Runner / SpawnFailed** (exit 1 — OS spawn error, no subprocess):
<!-- BUG-298: actual output missing [Runner] prefix — spawn_error_msg() emits bare "Failed to execute Claude Code: {e}" -->
```
Error: [Runner] failed to execute Claude Code: permission denied (os error 13) (exit 1)
```

**Runner / GateTimeout** (exit 1 — session gate exhausted, no subprocess):
```
Error: [Runner] session gate timed out — 30 active sessions, max-sessions=30 (exit 1)
```

**Unknown** (non-zero, no recognized pattern, exit ≤ 128, exit ≠ 2):
```
[Unknown] Unexpected internal error occurred. — retrying in 30s (attempt 1/2)…
Error: [Unknown] Unexpected internal error occurred. — retries exhausted (exit 1)
```
Without retry configured:
```
Error: [Unknown] Unexpected internal error occurred. (exit 1)
```
When subprocess emits no text:
```
Error: [Unknown] unknown error (exit 1)
```

### Cross-References

- [`docs/cli/type/13_error_kind.md`](13_error_kind.md) — runtime `ErrorKind` enum (subprocess variants)
- [`invariant/006_exit_codes.md`](../../invariant/006_exit_codes.md) — authoritative exit code table
- [`param/034_retry_on_transient.md`](../param/034_retry_on_transient.md) — Transient retry count
- [`param/035_transient_delay.md`](../param/035_transient_delay.md) — Transient delay
- [`param/040_retry_on_account.md`](../param/040_retry_on_account.md) — Account retry count
- [`param/041_account_delay.md`](../param/041_account_delay.md) — Account delay
- [`param/042_retry_on_auth.md`](../param/042_retry_on_auth.md) — Auth retry count
- [`param/043_auth_delay.md`](../param/043_auth_delay.md) — Auth delay
- [`param/044_retry_on_service.md`](../param/044_retry_on_service.md) — Service retry count
- [`param/045_service_delay.md`](../param/045_service_delay.md) — Service delay
- [`param/046_retry_on_process.md`](../param/046_retry_on_process.md) — Process retry count
- [`param/047_process_delay.md`](../param/047_process_delay.md) — Process delay
- [`param/048_retry_on_validation.md`](../param/048_retry_on_validation.md) — Validation retry count
- [`param/049_validation_delay.md`](../param/049_validation_delay.md) — Validation delay
- [`param/050_retry_on_runner.md`](../param/050_retry_on_runner.md) — Runner retry count
- [`param/051_runner_delay.md`](../param/051_runner_delay.md) — Runner delay
- [`param/052_retry_on_unknown.md`](../param/052_retry_on_unknown.md) — Unknown retry count
- [`param/053_unknown_delay.md`](../param/053_unknown_delay.md) — Unknown delay
- [`param/054_retry_override.md`](../param/054_retry_override.md) — Tier 1 override count
- [`param/055_retry_override_delay.md`](../param/055_retry_override_delay.md) — Tier 1 override delay
- [`param/056_retry_default.md`](../param/056_retry_default.md) — Tier 3 fallback count
- [`param/057_retry_default_delay.md`](../param/057_retry_default_delay.md) — Tier 3 fallback delay
- [`param/033_max_sessions.md`](../param/033_max_sessions.md) — Runner class gate configuration (unchanged)
- [`param/036_timeout.md`](../param/036_timeout.md) — Process class timeout threshold (unchanged)
- [`env_param.md`](../env_param.md) — complete env var mapping and precedence rules
