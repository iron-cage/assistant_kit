# Feature: Topic Pool

### Scope

- **Purpose**: Name a pool of anonymous topics — topics that exist to be somewhere for work to go, not to describe the work.
- **In Scope**: `DEFAULT_PREFIX`, `validate_prefix`, `pool_index`, `missing_names`.
- **Out of Scope**: Creating the topics the names identify (→ [`claude_runner`](../../../claude_runner/docs/cli/command/readme.md)); auto-naming a topic from the message that started it, which is descriptive by construction and belongs to `clr topic` (→ [`command/11_topic.md`](../../../claude_runner/docs/cli/command/11_topic.md)).

### Why This Exists

There are two ways to name a topic and they answer different questions.

An **auto-named** topic takes its name from the message that opened it — descriptive,
disambiguated by a collision counter, and meaningful to read back later. That is
what you want when the topic is *about* something.

A **pool** topic's name carries no meaning at all: `t1`, `t2`, `t3`. That is what
you want when you need four places to put work and do not yet know what the work
will be. Naming those after their first message would be actively misleading, since
the second message is unlikely to be about the same thing.

### Idempotence Is the Whole Design

"Make sure N topics exist" and "add N more topics" are different commands, and only
the first is usable from a script that may run twice. `missing_names` implements the
first: it reports what is absent, so running it again after a successful pass reports
nothing.

This is not a refinement. An implementation that appends N names looks correct on a
fresh base and is wrong on the second run — which is the run nobody is watching.

### Counting Rules

**Only pool-pattern names count toward the target.** A base holding ten richly-named
topics has zero pool topics, and asking for four gets four. Anything else would make
the meaning of `N` depend on unrelated work that happens to live in the same
directory.

**Gaps are filled before the range is extended.** With `t1` and `t3` present, a
target of four yields `t2` and `t4`, not `t4` and `t5`. A pool is a set of slots, and
a deleted topic leaves a slot rather than a permanent hole.

**One name per index, across both mechanisms.** A `t1` held in fork mode and a `t1`
held in dir mode are two topics ([002](002_topic_enumeration.md)) but one slot — the
caller creates one topic per missing *name*, and which mechanism it lands in is not
this module's decision.

**Asking for fewer than exist creates nothing.** It never deletes; `missing_names`
only ever reports absences.

### Prefix Rules

`pool_index` is the exact inverse of `format!( "{prefix}{index}" )`, and
`validate_prefix` rejects anything that would break that:

| Rejected | Why |
|----------|-----|
| Empty | Every name would be a bare number |
| Contains `/` | A topic name is a single path component, never a path |
| Contains a newline | The registry is one name per line |
| Starts with `-` | That prefix marks a topic directory |
| Ends in a digit | `t1` + index `2` is `t12`, which also reads as `t1` + index `2` the other way round — refusing the ambiguity is cheaper than resolving it |

`t01` is likewise not a pool name: a leading zero does not round-trip, so admitting
it would make the mapping many-to-one. Indices start at 1, so `t0` names nothing.

### Verification

```bash
cd module/claude_topic_core && ./verb/test
```

Or the single test binary, in-container:

```bash
cargo test -p claude_topic_core --test pool_test
```

tpl02 and tpl08 are the ones that matter — both are about the *second* run.

On a real base, idempotence is visible directly:

```bash
clr pool --count 4     # creates t1..t4
clr pool --count 4     # prints that nothing was missing; creates nothing
clr topics             # four rows, not eight
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/pool.rs` | Prefix rules, index parsing, top-up computation |
| doc | [002_topic_enumeration.md](002_topic_enumeration.md) | What "already exists" is read from |
| doc | [api/001_topic_surface.md](../api/001_topic_surface.md) | Full signature contract |
| test | `tests/pool_test.rs` | Idempotence, gap filling, prefix validation |
