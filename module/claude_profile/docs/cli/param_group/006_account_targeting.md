# Group: 6. Account Targeting

**Parameters:** `host::`, `role::`, `inference_provider::`
**Pattern:** Metadata labels attached to a saved account's profile
**Purpose:** Provides account-level metadata (machine/user context, role label, inference provider) that is stored in `{name}.json` at `.account.save` time and displayed via column projection in `.usage`/`.accounts`.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`host::`](../param/048_host.md) | `string` | `""` (auto: `$USER@$HOSTNAME`) | Machine/user label written to `{name}.json`; empty triggers auto-capture |
| [`role::`](../param/052_role.md) | `string` | `""` | User-defined role label written to `{name}.json`; persists across saves |
| [`inference_provider::`](../param/073_inference_provider.md) | `string` | *(omit; field absent — reads as `"anthropic"`)* | Inference provider label written to `{name}.json`; governs Gate 10 rotation grouping |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command-4-accountsave) | `host::`, `role::`, `inference_provider::` — write metadata to `{name}.json` |
| 2 | [`.accounts`](../command/001_account.md#command-3-accounts) | `host::` display toggle — opt-in boolean; `inference_provider` — default identity column, no dedicated toggle (opt-out via `cols::-inference_provider`) |

**Typical Patterns:**

```bash
# Auto-capture host from $USER@$HOSTNAME
clp .account.save

# Explicit host label
clp .account.save host::laptop

# Both host and role labels
clp .account.save host::workstation role::work

# Tag inference provider
clp .account.save inference_provider::kimi

# View stored metadata in usage table
clp .usage cols::+host,+role

# Hide the default-shown inference provider column
clp .accounts cols::-inference_provider
```

**Semantic Coherence Test**

> "Does parameter X attach a persistent metadata label to a saved account's profile?"

`host::` (param 048) passes: writes a human-readable machine/user label to `{name}.json`. `role::` (param 052) passes: writes a user-defined role label to `{name}.json`. `inference_provider::` (param 073) passes: writes an inference provider label to `{name}.json`. All other `.account.save` parameters fail — they store authentication data, not user-defined descriptive labels.

**Cross-References**

- [../../feature/029_account_host_metadata.md](../../feature/029_account_host_metadata.md) — feature spec for host/role metadata storage and display
- [../../feature/072_inference_provider_selection.md](../../feature/072_inference_provider_selection.md) — feature spec for inference provider metadata and global selection
- [../param/052_role.md](../param/052_role.md) — `role::` (metadata label) specification
- [../param/015_role.md](../param/015_role.md) — `role::` (field-presence toggle) for `.accounts` and `.credentials.status`
- [../param/048_host.md](../param/048_host.md) — `host::` parameter specification
- [../param/073_inference_provider.md](../param/073_inference_provider.md) — `inference_provider::` parameter specification
- [../param/033_cols.md](../param/033_cols.md) — `host`/`role` column IDs in `.usage`'s registry; `.accounts` has its own separate default identity set (including `inference_provider`) documented in [command/001_account.md](../command/001_account.md#command-3-accounts)

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | `host::`, `role::` metadata captured at account save |
