# Feature: Rich Account Metadata

### Scope

- **Purpose**: Expose rich account identity fields from `~/.claude.json` in `.credentials.status` and `.accounts` output.
- **Responsibility**: Documents the `display_name::`, `role::`, `billing::`, and `model::` field-presence params (FR-20).
- **In Scope**: Reading `oauthAccount` fields from `~/.claude.json`; reading `model` from `~/.claude/settings.json`; opt-in field-presence params `display_name::`, `role::`, `billing::`, `model::` on `.credentials.status` and `.accounts`; per-account metadata snapshots via `account::save()`.
- **Out of Scope**: Mutations to `~/.claude.json` (read-only); OAuth API calls; additional snapshot fields (`tagged_id`, `uuid`, `capabilities` → 021_extended_snapshot_fields.md); org identity from endpoint 005 (`org_uuid::`, `org_name::` → 022_org_identity_snapshot.md).

### Design

`.credentials.status` reads `emailAddress` from `~/.claude.json` via `read_live_cred_meta()`. This feature extends that read to expose additional `oauthAccount` fields and the active model setting — both on `.credentials.status` (live data) and `.accounts` (per-account snapshots).

**`.accounts` integration:** `account::save()` extracts the `oauthAccount` subtree from `~/.claude.json` into `{name}.json`. `account::list()` reads that snapshot to populate the `oauthAccount`-derived fields (`display_name`, `role`, `billing`) per account. The `model` field for `.accounts` is read from `{credential_store}/{name}.json` if present; returns `N/A` when that file is absent or lacks a `model` field. `save()` captures the current `model` from `~/.claude/settings.json` and writes it to `{credential_store}/{name}.json`; `switch_account()` reads this snapshot to restore or clear the `model` field in `~/.claude/settings.json` on each account switch (BUG-222 fix — see Feature 002 step 7, Feature 004 step 6).

**New field-presence params (all opt-in, default `0`):**

| Param | Default | Source | Output Line |
|-------|---------|--------|-------------|
| `display_name::` | `0` | `~/.claude.json → oauthAccount.displayName` | `Display: {displayName_or_N/A}` |
| `role::` | `0` | `.credentials.status`: `~/.claude.json → oauthAccount.organizationRole`; `.accounts`: `{name}.json → role` (user-defined label — see [029_account_host_metadata.md](029_account_host_metadata.md)) | `Role:    {value_or_N/A}` |
| `billing::` | `0` | `~/.claude.json → oauthAccount.billingType` | `Billing: {billingType_or_N/A}` |
| `model::` | `0` | `~/.claude/settings.json → model` | `Model:   {model_or_N/A}` |

**Why opt-in:** These fields are diagnostic/informational. The default output is already complete for routine use. Opt-in avoids cluttering the default view.

**`format::json`:** All new fields are included in JSON output regardless of field-presence params, extending the existing JSON shape with `"display_name"`, `"role"`, `"billing"`, `"model"` keys.

**Field schemas:** See [schema/007_claude_json.md](../schema/007_claude_json.md) for `~/.claude.json` fields and [schema/006_settings_json.md](../schema/006_settings_json.md) for `settings.json` fields.

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
- **AC-09**: `clp .accounts cols::+display_name` shows `Display:` line per account from saved `{name}.json` snapshot.
- **AC-10**: `clp .accounts cols::+role` shows `Role:` line with the **user-defined label** from saved `{name}.json` `role` field (Feature 029) — not `oauthAccount.organizationRole`; `cols::+billing` shows `Billing:` line from `oauthAccount.billingType`; `cols::+model` shows the value from `{name}.json` if present, or `N/A` if that file is absent or lacks a `model` field. `save()` writes the current model preference to `{name}.json` (BUG-222 fix); `switch_account()` restores it on account switch.
- **AC-11**: Accounts with no `{name}.json` snapshot on disk show `N/A` for `display_name`, `role`, `billing`, `model`.
- **AC-12**: `clp .accounts format::json` includes `display_name`, `role`, `billing`, `model` keys per account object.

### Bugs

| File | Relationship |
|------|--------------|
| BUG-222 | BUG-222 ✅ Fixed (TSK-234): `save()` now writes model to `{name}.json`; `switch_account()` restores/clears `model` in `~/.claude/settings.json` on each switch |

### Commands

| File | Relationship |
|------|--------------|
| [command/002_credentials.md](../cli/command/002_credentials.md#command--10-credentialsstatus) | CLI command specification |

### Features

| File | Relationship |
|------|--------------|
| [007_file_topology.md](007_file_topology.md) | `claude_json_file()` path method |
| [012_live_credentials_status.md](012_live_credentials_status.md) | Base `.credentials.status` command |
| [021_extended_snapshot_fields.md](021_extended_snapshot_fields.md) | Extends this feature: `uuid::`, `capabilities::` params |
| [022_org_identity_snapshot.md](022_org_identity_snapshot.md) | Extends this feature: `org_uuid::`, `org_name::` params via endpoint 005 |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/readme.md](../cli/param/readme.md) | New param entries (display_name::, role::, billing::, model::) |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.credentials.status`](../cli/command/002_credentials.md#command--10-credentialsstatus) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `module/claude_core/src/paths.rs` | `claude_json_file()` — path to `~/.claude.json` (FR-19) |
| `src/commands/credentials.rs`, `src/commands/accounts.rs` | `read_live_cred_meta()` — reads new fields; `credentials_status_routine()` — wires params; `accounts_routine()` — renders saved metadata |
| `src/lib.rs` | Registration of `display_name::`, `role::`, `billing::`, `model::` params |
| `claude_profile_core/src/account.rs` | `Account` struct with new fields; `save()` snapshots metadata files; `list()` reads snapshots |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/credentials_test.rs` | Test cases for each opt-in field on `.credentials.status` |
| `tests/cli/accounts_test.rs` | Test cases for rich metadata fields on `.accounts` |

### Schema

| File | Relationship |
|------|-------------|
| [schema/002_account_json.md](../schema/002_account_json.md) | Unified `{name}.json` field table — `oauthAccount`, `model` rows owned by this feature |
| [schema/007_claude_json.md](../schema/007_claude_json.md) | `~/.claude.json` fields read by this feature (`oauthAccount.displayName`, `organizationRole`, `billingType`) |
| [schema/006_settings_json.md](../schema/006_settings_json.md) | `model` field in `~/.claude/settings.json` captured at save time |
