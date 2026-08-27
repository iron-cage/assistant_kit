# CLI Parameter: --prefix

Names the pool [`pool`](../command/18_pool.md) fills, and therefore which existing
topics count toward its `--count`.

- **Type:** string
- **Default:** `t`
- **Command:** [`pool`](../command/18_pool.md)
- **Group:** None — `pool`'s own surface
- **JSON Key:** none (topic provisioning takes no JSON config)

```sh
clr pool 4                          # t1 t2 t3 t4
clr pool --prefix worker 8          # worker1 … worker8
clr pool --prefix review 2          # review1 review2
clr pool --dry-run --prefix w 3     # preview the names, create nothing
```

**A prefix names a pool, and pools are counted separately.** `--prefix worker`
does not see `t1`, and `--count` is compared only against topics matching the
prefix in force. So `clr pool --prefix worker 2` on a base already holding `t1` and
`t2` creates `worker1` and `worker2` — two independent pools, four topics. Verify:
`clr pool --dry-run --prefix worker 2 | grep '^existing:'` reports 0 on such a base.

**The rules exist to keep name → index one-to-one.** A pool name is exactly
`format!( "{prefix}{index}" )`, and the reverse direction has to be unambiguous or
the count is unreliable. `claude_topic_core::pool::validate_prefix` rejects five
shapes:

| Rejected | Why |
|----------|-----|
| Empty | Every name would be a bare number |
| Contains `/` | A topic name is a single path component, never a path |
| Contains a newline | The fork-topic registry is one name per line |
| Starts with `-` | That prefix marks a dir-mode topic directory |
| Ends in a digit | `t1` + index `2` is `t12`, which also reads as `t1` + index `2` the other way round |

The digit rule is the non-obvious one, and it is the reason the default is `t`
rather than something like `t1_`. Verify: `clr pool --prefix t1 2; echo $?` prints 1
with a reason naming the ambiguity.

**Leading zeros are not pool names.** `t01` does not round-trip — `format!` would
have produced `t1` — so it is neither counted as an existing pool topic nor ever
generated. Admitting it would make the mapping many-to-one and the count ambiguous.
Indices start at 1, so `t0` names nothing either.

**Changing the prefix does not rename anything.** It selects a different pool;
topics created under an old prefix stay exactly where they are and stop counting.
There is no rename operation — a pool topic's name is its identity, and in fork mode
it is also the input to the `UUIDv5` that locates its session.

**Validation:** rejected at parse time before the base is read, with the reason
quoted (`Error: invalid --prefix '<VALUE>': <reason>`). The rules belong to
`claude_topic_core::pool`, not to this CLI — they are properties of the
name-to-index mapping, and `clr` reports them rather than defining them.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| string | string | String | Non-empty; no `/`, no newline; not `-`-leading; not digit-trailing |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| — | None | `pool`-only | — |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 18 | [`pool`](../command/18_pool.md) | `t` | Passed to `validate_prefix()`, then to `pool_index()`/`missing_names()` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 33 | [033_topic_forwarding.md](../user_story/033_topic_forwarding.md) | Developer |
