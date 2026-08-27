# CLI Parameter: --count

How many pool topics must exist after [`pool`](../command/18_pool.md) runs.

- **Type:** usize (non-negative integer)
- **Default:** none — required
- **Command:** [`pool`](../command/18_pool.md)
- **Group:** None — `pool`'s own surface
- **JSON Key:** none (topic provisioning takes no JSON config)

```sh
clr pool --count 4                  # make sure four exist
clr pool 4                          # identical — the single positional is the count
clr pool --dry-run --count 4        # what it would create, for free
clr pool --count 0                  # accepted no-op
```

**A target, never an increment.** `--count 4` means "make sure four exist", not "add
four more". This is the difference between a command that is safe in a script that
may run twice and one that is not — and the second run is the one nobody is
watching. An implementation that appended `N` names would look correct on a fresh
base and silently double the pool on every rerun. Verify:

```sh
clr pool --dry-run 4 | grep '^create:'   # create: 4
clr pool 4
clr pool --dry-run 4 | grep '^create:'   # create: 0
```

**Counted against live pool topics only.** Two filters apply before the target is
compared. First, only pool-pattern names count — a base holding ten richly-named
topics has zero pool topics, so asking for four gets four. Second, only topics
holding at least one session count — a pool name whose session file was deleted is
missing and gets refilled, which is what keeps `clr pool --count 4 && clr broadcast`
reaching exactly four. Verify: `clr pool --dry-run 4 | grep '^existing:'` after
deleting one pool topic's `.jsonl` reports one fewer.

**Zero is accepted, not rejected.** `clr pool "$N"` from a script that computed
`N == 0` has asked for nothing, which is a thing to do. Failing there would force
every caller to special-case a value that already means "do nothing". Verify:
`clr pool --dry-run --count 0; echo $?` prints 0 with `create: 0` and no `cmd:`
lines.

**Asking for fewer than exist creates nothing, and deletes nothing.** The
computation only ever reports absences — `clr pool --count 2` on a base holding six
pool topics is a no-op, not a request to remove four. Deleting a topic is not
something a provisioning command should infer from a smaller number.

**Also accepted positionally, and only once.** `clr pool 4` and `clr pool --count 4`
produce byte-identical output. A second positional exits 1 suggesting `--message`:
`pool` takes a number, and reading a stray word as prose would hide the typo that
produced it. Verify: `clr pool 2 hello; echo $?` prints 1 naming `'hello'`.

**Validation:** a value that does not parse as `usize` is rejected at parse time
(`Error: count must be a non-negative integer, got '<VALUE>'`), and so is a missing
one (`Error: pool requires a count`). Negative values fail as unparsable — there is
no signed form.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| integer | usize | usize | Parses as a non-negative integer; `0` is a valid no-op |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| — | None | `pool`-only | — |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 18 | [`pool`](../command/18_pool.md) | required | Target passed to `claude_topic_core::pool::missing_names()` |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 33 | [033_topic_forwarding.md](../user_story/033_topic_forwarding.md) | Developer |
