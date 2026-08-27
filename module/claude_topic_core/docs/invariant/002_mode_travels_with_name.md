# Invariant: Mode Travels With Name

### Scope

- **Purpose**: Fix `( name, mode )` as the unit of topic addressing, so no consumer collapses two mechanisms into one row or forwards a name without the mode that resolves it.
- **Governs**: `Topic`, every function returning or accepting a topic, and every command that addresses a topic on a caller's behalf.
- **In Scope**: Why the pair is irreducible; what breaks when it is reduced.
- **Out of Scope**: The precedence rules themselves (→ [feature/001](../feature/001_topic_identity.md)); how the pair is rendered in CLI output.

### Rule

**A topic is a `( name, mode )` pair. A bare name does not identify a topic.**

`Topic` therefore carries `mode` as a field, `enumerate` emits one row per pair
rather than per name, and any command that hands a topic to another process must
pass `--topic-mode` alongside `--topic`.

**Rationale — the two mechanisms coexist by design.** Fork mode and dir mode leave
different traces in different places ([feature/002](../feature/002_topic_enumeration.md)),
and nothing prevents one name from being held by both. That is not a corner case to
be designed away: dir mode is what `--global` and `--from` require, and it is what
every topic created before fork mode existed already is.

**Rationale — reduction is silently lossy, not loudly wrong.** `effective_topic_mode`'s
rule 4 gives an existing `<base>/-<name>` directory priority over fork mode. So when
a name is held by both, a bare `--topic <name>`:

- always reaches the dir-mode topic,
- never reaches the fork-mode one,
- and reports nothing unusual while doing it.

An implementation that dedupes by name has not merged two views of one topic. It has
dropped a topic, and dropped the one that is harder to notice missing — the fork
topic has no directory to see in a file listing.

**Rationale — this binds consumers, not just this crate.** The failure lands one
layer up. A fan-out command that enumerates topics correctly and then forwards only
`--topic <name>` to each child re-introduces the bug at the boundary: every fork
topic in the list is silently redirected to its dir-mode twin, and the run looks
entirely successful. Passing the mode is what makes the enumeration mean anything.

### Verification

The mechanical checks are `ten04`, `ten05`, and `ten07` in
`tests/enumerate_test.rs` — one name in both mechanisms yields two rows with
different paths, they sort adjacently rather than collapsing, and only the fork row
carries a resumable session id — together with `tid13` in `tests/identity_test.rs`,
which asserts that an explicit `fork` reaches past an existing directory:

```bash
cd module/claude_topic_core && ./verb/test
```

By hand, the two halves of one name resolve to different places, and both sides are
inspectable:

```bash
clr topics --path review   # dir mode:  <base>/-review
clr topics --file review   # fork mode: <base storage>/<uuid>.jsonl
```

If both of those exist, the name is held twice — and `clr topics` will show two
rows for it. A listing that shows one row for a name whose `--path` and `--file`
targets both exist is this invariant being violated.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/enumerate.rs` | `Topic::mode`, one row per pair, sort by name then mode |
| source | `src/identity.rs` | The precedence that makes rule 4 outrank fork mode |
| doc | [feature/001_topic_identity.md](../feature/001_topic_identity.md) | The five precedence rules |
| doc | [feature/002_topic_enumeration.md](../feature/002_topic_enumeration.md) | Why the merge cannot dedupe by name |
| test | `tests/enumerate_test.rs` | ten04, ten05, ten07 |
| test | `tests/identity_test.rs` | tid13 — explicit mode reaches past an existing directory |
