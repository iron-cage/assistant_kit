# Parameter: 82. `tags::`

Comma-separated list of tags ([type/003](../../type/003_tag.md)). One name, three per-command roles — write the full tag set at save time, replace the tag set on `.account.tag`, and subset-filter the listing on `.accounts`.

- **Default:** *(omit)*
- **Constraints:** Comma-separated tags; each item lowercased then validated against `[a-z0-9_-]`, 1–64 chars; empty items rejected; duplicates collapsed; stored sorted. A failing item exits 1 naming the offending tag.
- **Purpose:** Partition the account fleet into overlapping named pools — the write and filter surface for the account-side half of tag-based selection ([feature/075](../../feature/075_account_tags.md)).

**Per-command behavior:**

| Command | Role |
|---------|------|
| `.account.save` | Writes the given set as the account's `tags` field; omitted → field untouched (absent ≡ empty set) |
| `.account.tag` | Replaces the account's whole tag set; mutually exclusive with `add::`/`remove::` |
| `.accounts` | Read-side filter: show only accounts whose tag set contains **all** listed tags |

**Examples:**

```bash
clp .account.save name::alice@acme.com tags::kimi_pool,ci
clp .account.tag name::alice@acme.com tags::personal        # replace whole set
clp .accounts tags::kimi_pool                                # only kimi_pool accounts
clp .accounts tags::kimi_pool,ci                             # accounts carrying BOTH tags
```

**Error cases:**

```text
clp .account.save tags::Kimi!Pool   → exit 1: invalid tag 'kimi!pool' (allowed: [a-z0-9_-], 1-64 chars)
clp .account.tag name::X tags::a add::b → exit 1: tags:: is mutually exclusive with add::/remove::
clp .account.tag name::X tags::a,,b → exit 1: empty tag item
```

**Notes:**
- Replace form deliberately reuses `tags::` rather than reviving the RETIRED `set::` ([param 055](055_set.md)) — one name, one concept.
- The first tag write to an account with a non-empty legacy `role` field triggers the lazy `role`→tag migration ([type/003](../../type/003_tag.md)).

**See Also:** [feature/075_account_tags.md](../../feature/075_account_tags.md) for the full tag design; [083_add.md](083_add.md)/[084_remove.md](084_remove.md) for incremental mutation.

### Referenced Type

- **Fundamental Type:** `string` (comma-separated [Tag](../../type/003_tag.md) list)

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Account Targeting](../param_group/006_account_targeting.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../command/001_account.md#command-4-accountsave) | Write full tag set at save time |
| 2 | [`.account.tag`](../command/001_account.md#command-25-accounttag) | Replace the whole tag set |
| 3 | [`.accounts`](../command/001_account.md#command-3-accounts) | Subset filter on the listing |

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Account Tags](../../feature/075_account_tags.md) | Owning feature — tag write, listing, and filter surface |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Partition the fleet into rotation pools |
