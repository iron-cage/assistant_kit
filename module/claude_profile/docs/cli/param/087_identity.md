# Parameter: 87. `identity::`

Target Identity (`USER@MACHINE`, [type/002](../../type/002_identity.md)) whose Tag Filter `.identity.filter` gets, sets, or clears. Omitted, the current Identity is targeted.

- **Default:** *(current Identity — `$USER@$HOSTNAME`, same resolution chain and sanitization as the active marker, [schema/005](../../schema/005_active_marker.md))*
- **Constraints:** `USER@MACHINE` form — exactly one `@`, both halves non-empty; sanitized into the filter filename per [schema/009](../../schema/009_identity_filter_json.md).
- **Purpose:** Central administration — set or inspect another seat's filter from any machine, since filter files are store-resident and sync with the credential store.

**Behavior:** All `.identity.filter` modes (get, set via `include::`/`exclude::`, delete via `clear::1`) operate on the file `_filter_{machine}_{user}` derived from this value instead of the caller's own.

**Examples:**

```text
clp .identity.filter identity::bob@laptop                      → show bob@laptop's filter
clp .identity.filter identity::bob@laptop include::kimi_pool   → pin bob@laptop to kimi_pool
clp .identity.filter identity::bob@laptop clear::1             → remove bob@laptop's filter
```

**Error cases:**

```text
clp .identity.filter identity::bob         → exit 1: identity:: must be USER@MACHINE
clp .identity.filter identity::@laptop     → exit 1: identity:: must be USER@MACHINE
```

**Notes:**
- Contrast with [`assignee::`](063_assignee.md) (same `USER@MACHINE` value form, different object: active-marker assignment) and [`owner::`](062_owner.md) (ownership field). `identity::` selects *whose filter*, never mutates markers or ownership.

**See Also:** [085_include.md](085_include.md), [086_exclude.md](086_exclude.md), [051_clear.md](051_clear.md).

### Referenced Type

- **Fundamental Type:** `string` (`USER@MACHINE`, [Identity](../../type/002_identity.md))

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.identity.filter`](../command/011_identity.md#command-24-identityfilter) | Select the target Identity |

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Identity Tag Filter](../../feature/076_identity_tag_filter.md) | Owning feature — cross-identity administration |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Administer another seat's rotation pool centrally |
