# Parameter: 85. `include::`

Comma-separated list of tags an account must **all** carry to be eligible for the target Identity's automatic selection. Sets the `include` half of the Identity's Tag Filter on `.identity.filter`.

- **Default:** *(omit — get mode; an absent filter file means empty include, i.e. no requirement)*
- **Constraints:** Same item rules as [`tags::`](082_tags.md); must not overlap `exclude::` (overlap exits 1 naming the tags). May be combined with `exclude::` in one invocation; mutually exclusive with `clear::`.
- **Purpose:** Pin an Identity's rotation pool to named tags — the positive half of the eligibility predicate `T ⊇ include ∧ T ∩ exclude = ∅` ([type/004](../../type/004_tag_filter.md)).

**Behavior:** The given set fully replaces the filter's `include` side (deduplicated, sorted) and is written to `_filter_{hostname}_{user}` ([schema/009](../../schema/009_identity_filter_json.md)). **Typo guard:** a write whose `include` matches zero currently-tagged accounts succeeds but warns on stderr — a typo here silently empties the rotation pool.

**Examples:**

```text
clp .identity.filter include::kimi_pool             → this seat rotates only within kimi_pool
clp .identity.filter include::kimi_pool,ci          → accounts must carry BOTH tags
clp .identity.filter include::ci exclude::personal  → both halves in one write
```

**Error cases:**

```text
clp .identity.filter include::a exclude::a   → exit 1: tag 'a' in both include and exclude
clp .identity.filter include::Bad!Tag        → exit 1: invalid tag 'bad!tag'
clp .identity.filter include::a clear::1     → exit 1: clear:: is mutually exclusive with include::/exclude::
```

**See Also:** [086_exclude.md](086_exclude.md) (negative half), [087_identity.md](087_identity.md) (cross-identity targeting).

### Referenced Type

- **Fundamental Type:** `string` (comma-separated [Tag](../../type/003_tag.md) list)

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.identity.filter`](../command/011_identity.md#command-24-identityfilter) | Set the filter's include set |

### Referenced Features

| # | Feature | Role |
|---|---------|------|
| 1 | [Identity Tag Filter](../../feature/076_identity_tag_filter.md) | Owning feature — predicate, Gate 11, typo guard |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Rotation](../user_story/001_account_rotation.md) | Pin a machine's rotation to a named pool |
