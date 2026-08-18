# Group: 6. Account Targeting

**Parameters:** `host::`, `tags::`, `inference_provider::`
**Pattern:** Metadata labels attached to a saved account's profile
**Purpose:** Provides account-level metadata (machine/user context, tag set, inference provider) that is stored in `{name}.json` at `.account.save` time and displayed via column projection in `.usage`/`.accounts`. The legacy `role::` label is removed — superseded by tags ([feature/075](../../feature/075_account_tags.md), 📋 planned).

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`host::`](../param/048_host.md) | `string` | `""` (auto: `$USER@$HOSTNAME`) | Machine/user label written to `{name}.json`; empty triggers auto-capture |
| [`tags::`](../param/082_tags.md) 📋 | `string` | *(omit — tag set unchanged)* | Comma-separated tag set written to `{name}.json`; supersedes the removed `role::` label |
| [`inference_provider::`](../param/073_inference_provider.md) | `string` | *(omit; field absent — reads as `"anthropic"`)* | Inference provider label written to `{name}.json`; governs Gate 10 rotation grouping |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command-4-accountsave) | `host::`, `tags::`, `inference_provider::` — write metadata to `{name}.json` |
| 2 | [`.accounts`](../command/001_account.md#command-3-accounts) | `host::` display toggle — opt-in boolean; `inference_provider` — default identity column, no dedicated toggle (opt-out via `cols::-inference_provider`); `tags::` — subset row filter (📋 planned) |
| 3 | [`.account.tag`](../command/001_account.md#command-25-accounttag) | `tags::` replace mode — dedicated tag mutation command (📋 planned) |

**Typical Patterns:**

```bash
# Auto-capture host from $USER@$HOSTNAME
clp .account.save

# Explicit host label
clp .account.save host::laptop

# Host label plus tags (📋 planned)
clp .account.save host::workstation tags::ci,work

# Tag inference provider
clp .account.save inference_provider::kimi

# View stored metadata in usage table
clp .usage cols::+host,+tags

# Hide the default-shown inference provider column
clp .accounts cols::-inference_provider
```

**Semantic Coherence Test**

> "Does parameter X attach a persistent metadata label to a saved account's profile?"

`host::` (param 048) passes: writes a human-readable machine/user label to `{name}.json`. `tags::` (param 082, 📋 planned) passes: writes the account's tag set to `{name}.json`. `inference_provider::` (param 073) passes: writes an inference provider label to `{name}.json`. All other `.account.save` parameters fail — they store authentication data, not user-defined descriptive labels.

**Cross-References**

- [../../feature/029_account_host_metadata.md](../../feature/029_account_host_metadata.md) — feature spec for host/role metadata storage and display
- [../../feature/072_inference_provider_selection.md](../../feature/072_inference_provider_selection.md) — feature spec for inference provider metadata and global selection
- [../param/082_tags.md](../param/082_tags.md) — `tags::` parameter specification (📋 planned)
- [../param/052_role.md](../param/052_role.md) — `role::` (metadata label), REMOVED — superseded by tags (Feature 075)
- [../../feature/075_account_tags.md](../../feature/075_account_tags.md) — feature spec for the tag set superseding role
- [../param/015_role.md](../param/015_role.md) — `role::` (field-presence toggle) for `.accounts` and `.credentials.status`
- [../param/048_host.md](../param/048_host.md) — `host::` parameter specification
- [../param/073_inference_provider.md](../param/073_inference_provider.md) — `inference_provider::` parameter specification
- [../param/033_cols.md](../param/033_cols.md) — `host`/`role` column IDs in `.usage`'s registry; `.accounts` has its own separate default identity set (including `inference_provider`) documented in [command/001_account.md](../command/001_account.md#command-3-accounts)

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding](../user_story/002_onboarding.md) | `host::`, legacy `role::` metadata captured at account save |
