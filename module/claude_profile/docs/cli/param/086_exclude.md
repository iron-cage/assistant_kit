# Parameter: 86. `exclude::`

Comma-separated list of tags an account must carry **none** of to be eligible for the target Identity's automatic selection. Sets the `exclude` half of the Identity's Tag Filter on `.identity.filter`.

- **Default:** *(omit — get mode; an absent filter file means empty exclude, i.e. nothing blocked)*
- **Constraints:** Same item rules as [`tags::`](082_tags.md); must not overlap `include::` (overlap exits 1 naming the tags). May be combined with `include::` in one invocation; mutually exclusive with `clear::`.
- **Purpose:** Steer an Identity's rotation away from named pools without enumerating everything else — the negative half of the eligibility predicate `T ⊇ include ∧ T ∩ exclude = ∅` ([type/004](../../type/004_tag_filter.md)).

**Behavior:** The given set fully replaces the filter's `exclude` side (deduplicated, sorted) and is written to `_filter_{hostname}_{user}` ([schema/009](../../schema/009_identity_filter_json.md)). Untagged accounts trivially pass any exclude (they carry nothing to intersect).

**Examples:**

```text
clp .identity.filter exclude::personal              → never rotate into personal accounts
clp .identity.filter exclude::personal,staging      → block both pools
clp .identity.filter include::ci exclude::personal  → both halves in one write
```

**Error cases:**

```text
clp .identity.filter exclude::a include::a   → exit 1: tag 'a' in both include and exclude
clp .identity.filter exclude::Bad!Tag        → exit 1: invalid tag 'bad!tag'
clp .identity.filter exclude::a clear::1     → exit 1: clear:: is mutually exclusive with include::/exclude::
```

**See Also:** [085_include.md](085_include.md) (positive half), [087_identity.md](087_identity.md) (cross-identity targeting).

### Referenced Type

- **Fundamental Type:** `string` (comma-separated [Tag](../../type/003_tag.md) list)

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.identity.filter`](../command/011_identity.md#command-24-identityfilter) | Set the filter's exclude set |

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Identity Tag Filter](../../feature/076_identity_tag_filter.md) | Owning feature — predicate, Gate 11 |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Keep specific pools out of a machine's rotation |
