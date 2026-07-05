# Parameter: 11. `file::`

Controls whether the credentials file path line appears in `.credentials.status` output. Opt-in (default `0`).

- **Default:** `0` (hidden)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`
- **Purpose:** Exposes the resolved path to `~/.claude/.credentials.json` for diagnostics and tooling integration.

**Examples:**

```text
file::0   → line omitted  (default)
file::1   → File:    /home/user/.claude/.credentials.json
```

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Field Presence](../param_group/002_field_presence.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.credentials.status`](../command/002_credentials.md#command-10-credentialsstatus) | Credentials file path line visibility |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Expose file path for tooling and diagnostics |
