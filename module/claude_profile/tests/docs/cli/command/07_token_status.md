# Test: `.token.status`

> **REMOVED** — `.token.status` was removed; its OAuth token expiry classification (`token::status_with_threshold()`) is now exposed via `.credentials.status`'s `Token:`/`Expires:` lines and `threshold::` parameter. See [command/002_credentials.md](../../../../docs/cli/command/002_credentials.md#command-10-credentialsstatus) and [command/005_token.md](../../../../docs/cli/command/005_token.md) (archived spec).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | N/A — command removed | Basic Invocation |
| IT-2 | N/A — command removed | Status Classification |
| IT-3 | N/A — command removed | Status Classification |
| IT-4 | N/A — command removed | Threshold Override |
| IT-5 | N/A — command removed | Threshold Edge |
| IT-6 | N/A — command removed | Output Format |
| IT-8 | N/A — command removed | Error Handling |
| IT-9 | N/A — command removed | Error Handling |

**Total:** 0 integration tests (8 superseded — see per-case notes)

**Source:** [feature/006_token_status.md](../../../../docs/feature/006_token_status.md), [command/002_credentials.md — .credentials.status](../../../../docs/cli/command/002_credentials.md#command-10-credentialsstatus)

---

### IT-1: N/A — command removed

> **N/A** — This case verified `clp .token.status` printed `valid — Xm remaining` for a far-future token. `.token.status` no longer exists; the underlying classification is exercised directly by `tests/docs/feature/006_token_status.md` FT-01 (`token::status()` API) and surfaced via `.credentials.status`'s `Token:`/`Expires:` lines (`docs/feature/012_live_credentials_status.md` Field Presence Table).
> Becomes testable when: no committed task.

---

### IT-2: N/A — command removed

> **N/A** — This case verified `clp .token.status` printed `expired` for a past-expiry token. Superseded by `tests/docs/feature/006_token_status.md` FT-01 (`status_returns_expired_when_expires_at_in_past`).
> Becomes testable when: no committed task.

---

### IT-3: N/A — command removed

> **N/A** — This case verified `clp .token.status` printed `expiring soon — Xm remaining` within the default threshold. Superseded by `tests/docs/feature/006_token_status.md` FT-02 (`status_returns_expiring_soon_within_default_threshold`); the CLI text differs under the new surface (`Token:   expiring in Xm`, not `expiring soon — Xm remaining` — see `docs/feature/012_live_credentials_status.md` Field Presence Table).
> Becomes testable when: no committed task.

---

### IT-4: N/A — command removed

> **N/A** — This case verified `clp .token.status threshold::1800` changed the classification boundary. `threshold::` now applies to `.credentials.status`; equivalent coverage lives in `tests/docs/cli/param/04_threshold.md` EC-3.
> Becomes testable when: no committed task.

---

### IT-5: N/A — command removed

> **N/A** — This case verified `threshold::0` never classifies as ExpiringSoon. Equivalent coverage lives in `tests/docs/cli/param/04_threshold.md` EC-2 (now exercised via `.credentials.status threshold::0`).
> Becomes testable when: no committed task.

---

### IT-6: N/A — command removed

> **N/A** — This case verified `format::json` returned `{"status":…,"expires_in_secs":N}`. `.credentials.status format::json` returns the same two fields (`"token"`, `"expires_in_secs"`) among its full 16-field object — see `tests/docs/cli/command/10_credentials_status.md` IT-3/IT-14 and `tests/docs/feature/006_token_status.md` FT-04.
> Becomes testable when: no committed task.

---

### IT-8: N/A — command removed

> **N/A** — This case verified a missing `~/.claude/.credentials.json` exited 2. Superseded by `tests/docs/cli/command/10_credentials_status.md` IT-4 (identical file-absence contract, same underlying error path — `require_claude_paths()`/file-existence check is shared code).
> Becomes testable when: no committed task.

---

### IT-9: N/A — command removed; behavior superseded, not equivalent

> **N/A** — This case verified an unparseable `expiresAt` exited 2. Under `.credentials.status`, this is no longer fatal: `derive_token_state()` (`src/commands/cmd_context.rs`) degrades gracefully on `Err`, rendering `Token: unknown` / `Expires: (unavailable)` while every other field still renders normally and the command exits 0 — see `docs/feature/006_token_status.md § Error handling`. There is no remaining code path where an unparseable `expiresAt` alone produces a non-zero exit; only a missing credentials file does (`tests/docs/cli/command/10_credentials_status.md` IT-4).
> Becomes testable when: no committed task.
