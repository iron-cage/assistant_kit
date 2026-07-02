# Feature: Retry Hierarchy

### Scope

- **Purpose**: Document the 3-tier retry resolution system for the 6 error classes that `clr` encounters during subprocess execution.
- **Responsibility**: Define how retry counts and delays are resolved per error class, and the relationship between tier-1 override, per-class budgets, and global fallback.
- **In Scope**: Retry count resolution (3 tiers), delay resolution (3 tiers), per-class error class mapping, retry budget semantics.
- **Out of Scope**: Exit code mapping for each error class (→ `invariant/006_exit_codes.md`), subprocess output parsing to classify errors (→ `claude_runner_core/src/types.rs`).

### Design

`clr` implements a 3-tier retry hierarchy that resolves the retry count and delay for each error class independently per invocation.

**Error Classes**

| Class | Trigger | Default Retries | Notes |
|-------|---------|-----------------|-------|
| Transient (`RateLimit`) | Exit 2, no quota message | 2 | Temporary rate limit |
| Account (`QuotaExhausted`) | Exit 2 + `"You've hit your limit"` | 2 | Quota exhausted |
| Auth | Subprocess auth failure | 2 | Same 3-tier resolution as all other classes |
| Service (`ApiError`) | Subprocess API error | 2 | Service-side transient error |
| Process (`Signal`) | Exit 128+N (killed by signal) | 2 | Process-level failure |
| Unknown | Any unclassified exit code | 2 | Safety net for unclassified errors |
| Validation | Exit 3 (`--expect` mismatch) | 0 | Controlled by `--retry-on-validation` |
| Runner | Exit 1 (CLR-layer error) | 0 | Controlled by `--retry-on-runner` |

**3-Tier Retry Count Resolution**

For each error class, retry count resolves in priority order:

1. **Tier 1 — Per-invocation override** (`--retry-override N`): If set, N applies to ALL error classes, overriding Tier 2 and Tier 3 unconditionally.
2. **Tier 2 — Per-class budget** (`--retry-on-{class} N`): If Tier 1 is not set, the per-class param governs retry count for that specific class.
3. **Tier 3 — Global fallback** (`--retry-default N`, default: 2): Applied when neither Tier 1 nor a matching Tier 2 param is configured.

**3-Tier Delay Resolution**

Delay between retries mirrors the same tier structure:

1. **Tier 1 delay** (`--retry-override-delay N`): Applied to all classes when `--retry-override` is in effect.
2. **Per-class delay** (`--{class}-delay N`, e.g., `--transient-delay 5`): Per-class delay for Tier 2.
3. **Global delay fallback** (`--retry-default-delay N`, default: 30): Applied when no higher-tier delay is set for the active class.

**Auth Retry Behavior (Fix BUG-325)**

Auth uses the same 3-tier retry resolution as all other classes (`--retry-override ?? --retry-on-auth ?? --retry-default`). The Tier-3 fallback default is 2 retries for Auth, identical to Transient/Account/Service/Process/Unknown. Set `--retry-on-auth 0` to explicitly disable Auth retry.

**Retry Budget Semantics**

The retry count is the number of ADDITIONAL attempts after the initial failure. A retry count of 2 means up to 3 total executions (1 initial + 2 retries). When the budget is exhausted, `clr` exits with the original exit code of the failed attempt.

### Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC-001 | `clr "msg" --retry-on-transient 5` retries up to 5 times on exit 2 (no quota text) |
| AC-002 | All error classes use the same 3-tier retry resolution; default count is 2 for all classes (Tier-3 fallback) |
| AC-003 | `clr "msg" --retry-override 0` disables retries for all error classes |
| AC-004 | `clr "msg" --retry-override 3` applies 3 retries to all classes, ignoring per-class settings |
| AC-005 | `clr "msg"` uses retry count 2 for Transient/Account/Auth/Service/Process/Unknown by default |
| AC-006 | `--retry-on-transient 3` with `--retry-override 0` → 0 retries (Tier 1 wins) |
| AC-007 | `--retry-default 5` sets the Tier 3 fallback to 5 for all classes without per-class overrides |
| AC-008 | Delay between retries resolves from `--retry-override-delay` → `--{class}-delay` → `--retry-default-delay` in tier priority order |

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](001_runner_tool.md) | Parent feature — execution modes that invoke retry logic |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/006_exit_codes.md](../invariant/006_exit_codes.md) | Exit code contract — exit codes that trigger retry classification |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/054_retry_override.md](../cli/param/054_retry_override.md) | Per-invocation retry count override (Tier 1) |
| [cli/param/055_retry_override_delay.md](../cli/param/055_retry_override_delay.md) | Per-invocation delay override (Tier 1) |
| [cli/param/056_retry_default.md](../cli/param/056_retry_default.md) | Global retry count fallback (Tier 3, default 2) |
| [cli/param/057_retry_default_delay.md](../cli/param/057_retry_default_delay.md) | Global delay fallback (Tier 3, default 30) |
| [cli/param/034_retry_on_transient.md](../cli/param/034_retry_on_transient.md) | Transient class retry budget (Tier 2) |
| [cli/param/035_transient_delay.md](../cli/param/035_transient_delay.md) | Transient class retry delay (Tier 2) |
| [cli/param/040_retry_on_account.md](../cli/param/040_retry_on_account.md) | Account class retry budget (Tier 2) |
| [cli/param/041_account_delay.md](../cli/param/041_account_delay.md) | Account class retry delay (Tier 2) |
| [cli/param/042_retry_on_auth.md](../cli/param/042_retry_on_auth.md) | Auth class retry budget (Tier 2, default auto) |
| [cli/param/043_auth_delay.md](../cli/param/043_auth_delay.md) | Auth class retry delay (Tier 2) |
| [cli/param/044_retry_on_service.md](../cli/param/044_retry_on_service.md) | Service class retry budget (Tier 2) |
| [cli/param/045_service_delay.md](../cli/param/045_service_delay.md) | Service class retry delay (Tier 2) |
| [cli/param/046_retry_on_process.md](../cli/param/046_retry_on_process.md) | Process class retry budget (Tier 2) |
| [cli/param/047_process_delay.md](../cli/param/047_process_delay.md) | Process class retry delay (Tier 2) |
| [cli/param/048_retry_on_validation.md](../cli/param/048_retry_on_validation.md) | Validation retry budget (Tier 2) |
| [cli/param/049_validation_delay.md](../cli/param/049_validation_delay.md) | Validation retry delay (Tier 2) |
| [cli/param/050_retry_on_runner.md](../cli/param/050_retry_on_runner.md) | Runner error retry budget (Tier 2) |
| [cli/param/051_runner_delay.md](../cli/param/051_runner_delay.md) | Runner error retry delay (Tier 2) |
| [cli/param/052_retry_on_unknown.md](../cli/param/052_retry_on_unknown.md) | Unknown class retry budget (Tier 2) |
| [cli/param/053_unknown_delay.md](../cli/param/053_unknown_delay.md) | Unknown class retry delay (Tier 2) |
