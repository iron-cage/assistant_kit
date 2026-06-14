# CLI Type: ErrorClass

Caller-facing taxonomy that groups the 12 observable CLI error conditions into 7 semantic classes.
Use this taxonomy to decide the appropriate caller response at the application layer.

- **Purpose:** Semantic grouping of CLI error conditions for caller response decisions
- **Fundamental Type:** taxonomy (7 classes, not an enum — used for documentation and caller guidance)
- **Constants:** see below
- **Constraints:** documentation type only; runtime classification uses `ErrorKind` (type 13) for subprocess errors
- **Parsing:** not a CLI input or runtime type — reference taxonomy for documentation
- **Methods:** —

### Error Class Table

| Class | Variants / Conditions | Retry? | Caller Action |
|-------|-----------------------|--------|---------------|
| **Transient** | `RateLimit` (exit 2, no text) | Yes — with backoff | Retry after `--retry-delay`; use `--retry-on-rate-limit` |
| **Account** | `QuotaExhausted` (`"You've hit your limit"`) | No — wait for reset | Wait for period reset or switch credentials |
| **Service** | `ApiError` (`"API Error: "`) | Maybe | Log and surface to user; may be server-side issue |
| **Auth** | `AuthError` (`"Your organization..."`) | No | Fix or rotate credentials |
| **Process** | `Signal` (exit > 128), Timeout (exit 2 + timeout stderr) | No | Investigate external signal source; increase `--timeout` |
| **Validation** | `ExpectMismatch` (exit 3) | Via `--expect-retries` | Check `--expect` pattern against actual output |
| **Runner** | `BinaryNotFound`, `SpawnFailed`, `GateTimeout`, `OutputFileError` (all exit 1) | No | Fix environment: install claude, fix paths, reduce session count |
| **Unknown** | `Unknown` (nonzero, no match, exit ≤ 128, exit ≠ 2) | Unknown | Surface raw stdout/stderr to user |

### Class Descriptions

**Transient** — Temporary rate-limiting by the Anthropic API (HTTP 429). The subprocess exits 2 with no distinguishing text. Automatic retry via `--retry-on-rate-limit N` with `--retry-delay SECS` cooldown is the correct response.

**Account** — Period quota exhausted for the current account. Distinguished from `Transient` by the `"You've hit your limit"` text in output; also exits 2. Retrying immediately is futile — wait for the billing period to reset or switch to a different account via `--creds`.

**Service** — API-layer error from the Anthropic backend (HTTP 4xx/5xx). The `"API Error: "` prefix (colon-space, not parenthesis) identifies these. May be transient infrastructure issues; surface to user for manual decision.

**Auth** — Credential or authorization failure. The subprocess rejects the current credentials. Rotating or re-issuing credentials is required.

**Process** — Subprocess died from an OS signal or was killed by the CLR timeout watchdog. `Signal` variants have exit code > 128; `Timeout` variants have exit 2 plus the `"Error: timeout after {N}s"` stderr line. Increasing `--timeout` or investigating external process killers is the response.

**Validation** — Output did not match the `--expect` pattern within the allowed `--expect-retries` count. The CLR layer exits 3. Adjust the pattern or increase retry count.

**Runner** — CLR infrastructure failure before or after subprocess execution: binary not found in PATH, OS spawn error, session gate timed out waiting for a slot, or output file write failure. All exit 1. Fix the environment rather than retrying.

**Unknown** — Non-zero exit with no recognized pattern, exit code not 2, exit code ≤ 128. Surface raw stdout/stderr; investigate subprocess logs.

### Exit Code to Class Mapping

| Exit Code | Possible Classes | Disambiguation |
|-----------|-----------------|----------------|
| 0 | — | Success (no class) |
| 1 | Runner | BinaryNotFound / SpawnFailed / GateTimeout / OutputFileError |
| 2 | Transient, Account, Process | Stderr contains `"timeout after"` → Process; text contains `"hit your limit"` → Account; otherwise → Transient |
| 3 | Validation | ExpectMismatch (CLR layer) |
| > 128 | Process | Signal (POSIX 128+N convention) |
| other | Unknown, Auth, Service | Check stdout/stderr text for `"API Error: "` or `"Your organization"` |

### Strategy Configuration per Class

| Class | Default Strategy | Strategy Options | Existing Parameters | Proposed Parameters | Status |
|-------|-----------------|------------------|---------------------|---------------------|--------|
| **Transient** | retry 1, delay 30s | abort (retry=0), retry N with delay | `--retry-on-rate-limit` (u8, default 1), `--retry-delay` (u32, default 30s) | — | Complete |
| **Account** | abort | abort, switch | — | `--on-quota-exhausted` (abort\|switch) | Gap |
| **Service** | abort (retry=0) | abort (retry=0), retry N with delay | — | `--retry-on-api-error` (u8, default 0), `--api-error-delay` (u32, default 30s) | Gap |
| **Auth** | abort | abort, switch | — | `--on-auth-error` (abort\|switch) | Gap |
| **Process** | abort | abort | `--timeout` (u32, default 0=unlimited) | — | Complete (Timeout threshold configurable; Signal is not retryable) |
| **Validation** | fail | fail, retry, default:\<V\> | `--expect-strategy` (enum, default fail), `--expect-retries` (u8, default 0) | — | Complete |
| **Runner** | abort | abort | `--max-sessions` (u32, default 30) | — | Complete (gate threshold configurable; BinaryNotFound/SpawnFailed are not retryable) |
| **Unknown** | abort (retry=0) | abort (retry=0), retry N | — | `--retry-on-unknown-error` (u8, default 0) | Gap |

**Strategy Coverage:** 4 of 8 classes have configurable strategy (Transient, Process, Validation, Runner). 4 classes default to hard-coded abort with no configuration (Account, Service, Auth, Unknown).

**Proposed new parameters** (5 total, all following the existing naming convention):

| # | CLI Parameter | Env Var | Type | Default | Error Class |
|---|--------------|---------|------|---------|-------------|
| 37 | `--on-quota-exhausted` | `CLR_ON_QUOTA_EXHAUSTED` | enum (abort\|switch) | abort | Account |
| 38 | `--retry-on-api-error` | `CLR_RETRY_ON_API_ERROR` | u8 | 0 | Service |
| 39 | `--api-error-delay` | `CLR_API_ERROR_DELAY` | u32 (seconds) | 30 | Service |
| 40 | `--on-auth-error` | `CLR_ON_AUTH_ERROR` | enum (abort\|switch) | abort | Auth |
| 41 | `--retry-on-unknown-error` | `CLR_RETRY_ON_UNKNOWN_ERROR` | u8 | 0 | Unknown |

**Note:** The `switch` option for Account and Auth classes requires multi-credential infrastructure — a `--creds` list or directory from which `clr` can select an alternate credential file after an account exhaustion or auth failure. This is a prerequisite dependency, not a simple parameter addition.

### Configuration Tiers

Current system uses 3 tiers. Target is 4 tiers with a config file layer.

| Priority | Tier | Source | Status |
|----------|------|--------|--------|
| 1 (highest) | CLI parameter | `--flag value` on command line | Exists (36 params) |
| 2 | Environment variable | `CLR_*` env vars | Exists (39 vars) |
| 3 | Config file | TBD — `~/.config/clr/config.toml` or `$CLR_CONFIG` | Gap |
| 4 (lowest) | Hardcoded default | Built into source code | Exists |

**Config file gap:** No config file mechanism exists. Config keys would use `snake_case` matching CLI `--kebab-case` names (e.g., `retry_on_rate_limit = 1`). All 36+ parameters should be configurable at the config file tier.

### Cross-References

- [`docs/cli/type/13_error_kind.md`](13_error_kind.md) — runtime `ErrorKind` enum (subprocess variants)
- [`invariant/006_exit_codes.md`](../../invariant/006_exit_codes.md) — authoritative exit code table
- [`param/034_retry_on_rate_limit.md`](../param/034_retry_on_rate_limit.md) — automatic Transient retry
- [`param/035_retry_delay.md`](../param/035_retry_delay.md) — retry cooldown
- [`param/032_expect_retries.md`](../param/032_expect_retries.md) — Validation class retry count
- [`param/033_max_sessions.md`](../param/033_max_sessions.md) — Runner class gate configuration
- [`env_param.md`](../env_param.md) — complete env var mapping and precedence rules
