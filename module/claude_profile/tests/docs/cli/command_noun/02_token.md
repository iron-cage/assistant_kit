# Test: noun::token

> **REMOVED** — `.token.status` (the only command on this noun) has been removed. Token expiry classification is now exposed via `.credentials.status`'s `token`/`expires` fields. See [docs/cli/command_noun/002_token.md](../../../../docs/cli/command_noun/002_token.md) (archived) and [docs/cli/command_noun/003_credentials.md](../../../../docs/cli/command_noun/003_credentials.md).

Noun contract tests for the `token` domain noun. Verifies stateless read behavior,
JSON output schema fidelity, and error code contract as defined in
[docs/cli/command_noun/002_token.md](../../../../docs/cli/command_noun/002_token.md).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| NC-1 | N/A — command removed | Lifecycle |
| NC-2 | N/A — command removed | Output Schema |
| NC-3 | N/A — command removed | Error Code Contract |

**Total:** 0 noun contract tests (3 superseded — see per-case notes)

---

### NC-1: N/A — command removed

> **N/A** — This case verified `.token.status` wrote no persistent state (mtime-stable, pure read). `.credentials.status` is documented as a pure read with the same guarantee (`docs/cli/command_verb/010_status.md` Post-conditions: "No files written or modified"; `tests/docs/cli/command/10_credentials_status.md` IT-8 confirms output stability across repeated invocations). No dedicated mtime-check case exists under the new noun; the property is covered structurally rather than by a standalone test.
> Becomes testable when: no committed task.

---

### NC-2: N/A — command removed

> **N/A** — This case verified `.token.status format::json`'s two-field schema (`status`, `expires_in_secs`). Superseded by `tests/docs/cli/command/10_credentials_status.md` IT-3/IT-14, which assert the same two fields (`token`, `expires_in_secs`) as part of `.credentials.status`'s full 16-field JSON object.
> Becomes testable when: no committed task.

---

### NC-3: N/A — command removed

> **N/A** — This case verified a missing credentials file exited 2 with no stdout. Superseded by `tests/docs/cli/command/10_credentials_status.md` IT-4 (identical file-absence contract).
> Becomes testable when: no committed task.
