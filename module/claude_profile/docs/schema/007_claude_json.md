# Schema: Claude State — `~/.claude.json`

### Scope

- **Purpose**: Define which fields in `~/.claude.json` are read by `clp`, their locations, and read callers.
- **Responsibility**: Documents the `~/.claude.json` fields read by `clp` and their read callers.
- **In Scope**: All `~/.claude.json` fields that `clp` reads — `oauthAccount` subtree and any auxiliary fields.
- **Out of Scope**: Full `~/.claude.json` schema (not owned by clp — it is owned exclusively by the Claude binary). `clp` NEVER writes to this file.

### File Location

```
~/.claude.json
```

Note: This file is a **sibling** to `~/.claude/` (the directory), not inside it. Path via `ClaudePaths::claude_json_file()`. See [schema/003](003_file_topology.md).

### Read-Only Contract

`clp` never writes to `~/.claude.json`. Mutations to this file are owned by the Claude binary (OAuth login, session state). All `clp` reads are graceful — absent file or absent fields show `N/A` without error.

### Fields Read by `clp`

| JSON path | Type | Semantics | Read by | Feature |
|-----------|------|-----------|---------|---------|
| `oauthAccount.emailAddress` | string | Primary email address — used as default account name during `.account.save` when `name::` is omitted | `read_live_cred_meta()`, `account_save_routine()` | [002](../feature/002_account_save.md), [012](../feature/012_live_credentials_status.md) |
| `oauthAccount.displayName` | string | Human-readable display name | `read_live_cred_meta()` | [014](../feature/014_rich_account_metadata.md) |
| `oauthAccount.organizationRole` | string | Organization role (`admin`, `member`, etc.) | `read_live_cred_meta()` | [014](../feature/014_rich_account_metadata.md) |
| `oauthAccount.billingType` | string | Billing type (`stripe_subscription`, etc.) | `read_live_cred_meta()` | [014](../feature/014_rich_account_metadata.md) |
| `oauthAccount.id` | string | Unique account UUID | `save()` for `{name}.json` snapshot | [021](../feature/021_extended_snapshot_fields.md) |
| `oauthAccount.primaryEmailAddress` | string | Primary email (used for `tagged_id` when present) | `save()` for `{name}.json` snapshot | [021](../feature/021_extended_snapshot_fields.md) |
| `oauthAccount.capabilities` | array of strings | Account capability flags (e.g., `"claude_max"`) | `save()` for `{name}.json` snapshot | [021](../feature/021_extended_snapshot_fields.md) |
| `oauthAccount.subscriptionType` | string | Subscription type (`max`, `pro`, other) — mapped to login method label | `read_live_cred_meta()` | [014](../feature/014_rich_account_metadata.md) |

### Example Layout

```json
{
  "oauthAccount": {
    "emailAddress": "alice@example.com",
    "primaryEmailAddress": "alice@example.com",
    "displayName": "Alice",
    "organizationRole": "admin",
    "billingType": "stripe_subscription",
    "subscriptionType": "max",
    "id": "01234567-89ab-cdef-0123-456789abcdef",
    "capabilities": ["claude_max"]
  }
}
```

### Graceful Missing-Field Handling

All fields show `N/A` when:
- File is absent
- `oauthAccount` key is missing
- Specific sub-field is missing or empty string

No error is ever raised for absent metadata in `~/.claude.json`.

### Features

| File | Relationship |
|------|-------------|
| [feature/012_live_credentials_status.md](../feature/012_live_credentials_status.md) | `read_live_cred_meta()` — live credential metadata reading |
| [feature/014_rich_account_metadata.md](../feature/014_rich_account_metadata.md) | `oauthAccount` fields: `displayName`, `organizationRole`, `billingType` |
| [feature/021_extended_snapshot_fields.md](../feature/021_extended_snapshot_fields.md) | `id`, `primaryEmailAddress`, `capabilities` |

### Schema

| File | Relationship |
|------|-------------|
| [002_account_json.md](002_account_json.md) | `oauthAccount` subtree is snapshotted into `{name}.json` at save time |
| [003_file_topology.md](003_file_topology.md) | `claude_json_file()` path method |
