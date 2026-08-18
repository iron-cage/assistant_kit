# Parameter: 84. `remove::`

Comma-separated list of tags to remove from an account's tag set on `.account.tag`.

- **Default:** *(omit)*
- **Constraints:** Same item rules as [`tags::`](082_tags.md) — lowercased, `[a-z0-9_-]`, 1–64 chars, no empty items. Mutually exclusive with `tags::` and with `add::`.
- **Purpose:** Take an account out of one pool without touching its other tags.

**Behavior:** Listed tags are removed from the existing set; removing a tag the account does not carry is a no-op success (idempotent — safe in scripts). The result stays sorted. The first-tag-write `role`→tag migration also fires on a `remove::` write ([type/003](../../type/003_tag.md)).

**Examples:**

```text
clp .account.tag name::alice@acme.com remove::ci        → 'ci' removed
clp .account.tag name::alice@acme.com remove::absent    → no-op, exit 0
clp .account.tag name::X,Y remove::staging              → comma-list batch
```

**Error cases:**

```text
clp .account.tag name::X remove::ci add::ci   → exit 1: one operation per invocation
clp .account.tag name::X remove::Bad!Tag      → exit 1: invalid tag 'bad!tag'
```

**See Also:** [083_add.md](083_add.md) (inverse operation), [082_tags.md](082_tags.md) (replace form).

### Referenced Type

- **Fundamental Type:** `string` (comma-separated [Tag](../../type/003_tag.md) list)

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.tag`](../command/001_account.md#command-25-accounttag) | Remove tags from the account's set |

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Tags](../../feature/075_account_tags.md) | Owning feature |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Remove an account from a rotation pool |
