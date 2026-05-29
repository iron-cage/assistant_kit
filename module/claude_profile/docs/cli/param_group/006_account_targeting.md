# Group :: 6. Account Targeting

**Parameters:** `host::`
**Pattern:** Metadata labels attached to a saved account's profile
**Purpose:** Provides account-level metadata (machine/user context, role label) that is stored in `{name}.profile.json` at `.account.save` time and displayed via column projection in `.usage`.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`host::`](../param/048_host.md) | `string` | `""` (auto: `$USER@$HOSTNAME`) | Machine/user label written to `profile.json`; empty triggers auto-capture |

**Note:** `.account.save` also accepts the pre-existing [`role::`](../param/015_role.md) parameter (param 015) as a string label for the account's role context (e.g., `"work"`, `"dev"`). On `.account.save`, `role::` takes a free-form string value; on `.credentials.status`, `role::` is a boolean field-presence toggle — the semantics are command-scoped.

**Used By (1 command):** [`.account.save`](../command/001_account.md#command--4-accountsave)

**Typical Patterns:**

```bash
# Auto-capture host from $USER@$HOSTNAME
clp .account.save

# Explicit host label
clp .account.save host::laptop

# Both host and role labels
clp .account.save host::workstation role::work

# View stored metadata in usage table
clp .usage cols::+host,+role
```

**Semantic Coherence Test**

> "Does parameter X attach a persistent metadata label to a saved account's profile?"

`host::` passes: it writes a human-readable machine/user label to `{name}.profile.json` that persists across saves. `role::` (param 015 on `.account.save`) also passes when used with that command. All other `.account.save` parameters (the credential/token fields) fail — they store authentication data, not user-defined descriptive labels.

**Cross-References**

- [../../feature/029_account_host_metadata.md](../../feature/029_account_host_metadata.md) — feature spec for host/role metadata storage and display
- [../param/015_role.md](../param/015_role.md) — `role::` dual-use: Account Targeting (`.account.save`) and Field Presence (`.credentials.status`)
- [../param/048_host.md](../param/048_host.md) — `host::` parameter specification
- [../param/033_cols.md](../param/033_cols.md) — `host` and `role` column IDs in `.usage`
