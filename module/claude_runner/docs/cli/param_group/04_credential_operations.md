# CLI Parameter Group: Credential Operations

**Pattern:** Shared by `clr isolated` and `clr refresh`; configure the credential-isolated execution environment; not accepted by `clr run`.

**Purpose:** Configure credential-isolated execution for `clr isolated` and `clr refresh`.
**Order:** 4

### Semantic Coherence Test

"Is this parameter used by credential-operating commands (`isolated`/`refresh`) and not by `run`?" — YES for all 3.

### Why NOT X

- `--creds`: exclusive to credential ops; sets credentials file — not applicable to `run`
- `--timeout`: exclusive to credential ops; controls subprocess wait time — not applicable to `run`
- `--trace`: also in Runner Control (Group 2) for `run`; listed here because it applies to credential ops too

### Invariants

`--creds` and `--timeout` are exclusive to `clr isolated` and `clr refresh`. Neither is accepted by `clr run`. `--trace` is cross-command (also in Group 2).

### Notes

`--timeout` has different defaults per command: 30s for `isolated` (general task execution), 45s for `refresh` (allows headroom for slow OAuth token exchange).

### Typical Patterns

```sh
clr isolated "Fix bug"                                    # uses ~/.claude/.credentials.json
clr isolated --creds ~/.claude/.credentials.json "Fix bug"
clr isolated --creds /path/to/creds.json --timeout 120 --trace "Refactor this"
clr refresh                                               # uses ~/.claude/.credentials.json
clr refresh --creds creds.json --timeout 90 --trace
```

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|------------|-----------------|-------|
| 2 | [`isolated`](../command/02_isolated.md) | Full | — | All 3 params apply |
| 3 | [`refresh`](../command/03_refresh.md) | Full | — | All 3 params apply |

### Referenced Parameters

| Parameter | Type | Default | Role in Group | Description |
|-----------|------|---------|---------------|-------------|
| [`--creds`](../param/019_creds.md) | [`CredentialsFilePath`](../type/08_credentials_file_path.md) | `~/.claude/.credentials.json` | Credentials source | Credentials JSON file (optional; defaults to current account credentials) |
| [`--timeout`](../param/020_timeout.md) | [`TimeoutSecs`](../type/09_timeout_secs.md) | 30/45 | Duration limit | Max seconds to wait (30 isolated, 45 refresh) |
| [`--trace`](../param/013_trace.md) | bool | false | Trace mode | Print underlying call details to stderr then execute |

### Referenced Tests

| # | Test Spec | Scope |
|---|-----------|-------|
| 4 | [04_credential_operations.md](../../../tests/docs/cli/param_group/04_credential_operations.md) | Credential Operations group behavior |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 8 | [008_trace_execution.md](../user_story/008_trace_execution.md) | Developer |
| 10 | [010_credential_isolated_execution.md](../user_story/010_credential_isolated_execution.md) | Developer |
| 14 | [014_credential_refresh.md](../user_story/014_credential_refresh.md) | Developer |
