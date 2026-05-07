# Feature: Rich Account Metadata

### Scope

- **Purpose**: Expose rich account identity fields from `~/.claude.json` in `.credentials.status` output.
- **Responsibility**: Documents the `display_name::`, `role::`, `billing::`, and `model::` field-presence params (FR-20).
- **In Scope**: Reading `oauthAccount` fields from `~/.claude.json`; reading `model` from `~/.claude/settings.json`; new opt-in field-presence params on `.credentials.status`.
- **Out of Scope**: Mutations to `~/.claude.json` (read-only); per-saved-account metadata enrichment (separate feature); OAuth API calls.

### Design

`.credentials.status` currently reads `emailAddress` and `organizationName` from `~/.claude.json` via `read_live_cred_meta()`. Those two fields work correctly once the path bug (FR-19) is fixed. This feature extends that read to expose additional `oauthAccount` fields and the active model setting.

**New field-presence params (all opt-in, default `0`):**

| Param | Default | Source | Output Line |
|-------|---------|--------|-------------|
| `display_name::` | `0` | `~/.claude.json → oauthAccount.displayName` | `Display: {displayName_or_N/A}` |
| `role::` | `0` | `~/.claude.json → oauthAccount.organizationRole` | `Role:    {organizationRole_or_N/A}` |
| `billing::` | `0` | `~/.claude.json → oauthAccount.billingType` | `Billing: {billingType_or_N/A}` |
| `model::` | `0` | `~/.claude/settings.json → model` | `Model:   {model_or_N/A}` |

**Why opt-in:** These fields are diagnostic/informational. The default output is already complete for routine use. Opt-in avoids cluttering the default view.

**`format::json`:** All new fields are included in JSON output regardless of field-presence params, extending the existing JSON shape with `"display_name"`, `"role"`, `"billing"`, `"model"` keys.

**`~/.claude.json` layout (relevant fields):**
```json
{
  "oauthAccount": {
    "displayName": "alice",
    "organizationName": "Acme Corp",
    "organizationRole": "admin",
    "billingType": "stripe_subscription"
  }
}
```

**`~/.claude/settings.json` layout (relevant field):**
```json
{ "model": "sonnet" }
```

**Missing or empty fields:** All new fields show `N/A` when the source file is absent or the field is missing/empty. Never error on absent metadata — `.credentials.status` is a graceful read command.

**Login method label:** `subscriptionType` values map to human-readable labels matching Claude Code's `/status` display:

| `subscriptionType` | Login Method Label |
|--------------------|--------------------|
| `max` | `Claude Max Account` |
| `pro` | `Claude Pro Account` |
| *(other)* | *(raw value)* |

This label is NOT a separate field param — it is the formatted output of the existing `sub::` field when `format::text` is used. The `format::json` key remains `"subscription"` with the raw value.

### Acceptance Criteria

- **AC-01**: `clp .credentials.status display_name::1` shows `Display: {displayName}` or `Display: N/A`.
- **AC-02**: `clp .credentials.status role::1` shows `Role: {organizationRole}` or `Role: N/A`.
- **AC-03**: `clp .credentials.status billing::1` shows `Billing: {billingType}` or `Billing: N/A`.
- **AC-04**: `clp .credentials.status model::1` shows `Model: {model}` or `Model: N/A`.
- **AC-05**: All four params default to `0` — absent from default `.credentials.status` output.
- **AC-06**: `format::json` includes `display_name`, `role`, `billing`, `model` keys regardless of field-presence params.
- **AC-07**: Absent `~/.claude.json` → all three oauthAccount fields show `N/A` without error.
- **AC-08**: Absent `~/.claude/settings.json` → `model` shows `N/A` without error.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `module/claude_core/src/paths.rs` | `claude_json_file()` — path to `~/.claude.json` (FR-19) |
| source | `src/commands.rs` | `read_live_cred_meta()` — reads new fields; `credentials_status_routine()` — wires params |
| source | `src/lib.rs` | Registration of `display_name::`, `role::`, `billing::`, `model::` params |
| test | `tests/cli/credentials_test.rs` | New test cases for each opt-in field |
| doc | [007_file_topology.md](007_file_topology.md) | `claude_json_file()` path method |
| doc | [012_live_credentials_status.md](012_live_credentials_status.md) | Base `.credentials.status` command |
| doc | [cli/commands.md](../cli/commands.md#command--10-credentialsstatus) | CLI command specification |
| doc | [cli/params.md](../cli/params.md) | New param entries (display_name::, role::, billing::, model::) |
