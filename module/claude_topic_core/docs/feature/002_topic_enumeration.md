# Feature: Topic Enumeration

### Scope

- **Purpose**: Answer "which topics exist under this base?" across two mechanisms that leave completely different traces, and separate the ones that hold a conversation from the ones that merely have a name.
- **In Scope**: `Topic`, `Topic::session_id`, `session_count`, `enumerate`, `enumerate_live`, and the `registry` module that makes fork-mode names visible at all.
- **Out of Scope**: What to do with the results — choosing one (→ [003](003_topic_selection.md)), creating missing ones (→ [004](004_topic_pool.md)); the encoding that maps a base path to a storage directory (→ [`claude_storage_core`](../../../claude_storage_core/docs/readme.md)).

### Why This Exists

The two mechanisms have to be merged, and merging them is not a formality:

- A **dir-mode** topic is a `<base>/-<name>` directory. Scanning the base finds it.
- A **fork-mode** topic leaves *nothing in the base at all* — only a `UUIDv5`-named
  session file in the base's own storage. `UUIDv5` is one-way, so the name cannot
  be recovered from the file that embodies the topic.

That asymmetry is the whole reason the registry exists.

### The Registry

One plain-text file per base directory, under `$CLR_TOPIC_REGISTRY_DIR` or
`~/.clr/topics/`, named by the base's storage encoding, holding one topic name per
line. Recording is append-if-missing and warn-never-fatal: a failed index write must
not break the run that triggered it.

It is a **convenience index, never an authority** — see
[invariant/001](../invariant/001_registry_non_authoritative.md). Two consequences
follow directly from the file format:

- Entries outlive the sessions they name. A registry entry whose session file was
  deleted still lists, with `sessions == 0`.
- **A name containing a newline is never recorded.** Such a topic works fine as a
  session; it simply cannot be listed, and therefore cannot be reached by any
  command that enumerates rather than being handed the name.

### The Unit Is a Pair

`Topic` carries `name`, `mode`, `path`, and `sessions`. The unit of enumeration is
`( name, mode )` and never the name alone, because both mechanisms can legitimately
hold the same name at once.

Collapsing them into one row does not lose an edge case; it loses a topic. And by
[001](001_topic_identity.md)'s rule 4, the one it loses is always the fork — the
directory always wins a bare address.

`session_id()` is `Some` only for a fork topic: that is the deterministic id
`claude --resume` takes, and the key selection and locking recognise in a live
process's argv. A dir topic's sessions carry ordinary Claude-generated ids that no
formula predicts, so it is `None`.

### `enumerate` vs `enumerate_live`

`enumerate` reports everything found, sorted by name and then by mode (so `dir`
sorts before `fork`, keeping a name's two mechanisms adjacent).

`enumerate_live` keeps only topics with `sessions > 0`. That filter does two jobs:

**1. It is the difference between continuing a conversation and starting one.** A
registry entry whose file was deleted, or a `-name/` directory nobody has ever run
in, has no conversation to continue — addressing it *creates* one by forking the
base. For a command that fans a prompt out over "my topics", silently minting new
conversations is the wrong reading of the request.

**2. It keeps fan-out out of non-topic directories.** `topic_name_of` accepts *any*
`-`-prefixed directory, and this workspace marks generated and ignored directories
exactly the same way — `-daemon/`, `-gate/`, and every `./-NNNN_*` scratch path look
like dir-mode topics from the base's point of view. They have no session storage, so
the filter drops them.

Treat the second as a **strong heuristic and not a guarantee**: a scratch directory
someone did once run `claude` inside genuinely does have storage, and genuinely will
be enumerated. Any command that acts on every topic should show what it resolved
before acting on it.

### Session Counting

A dir topic's count is the number of `*.jsonl` files in that directory's own session
storage — zero for a topic directory created but never entered, since the storage
directory is made by Claude Code on first run and not by whoever made the topic.

A fork topic's count is 0 or 1, because a fork topic *is* one session file. An
existing but zero-length file counts as 0: a file is not a conversation.

### Verification

```bash
cd module/claude_topic_core && ./verb/test
```

Or the single test binary, in-container:

```bash
cargo test -p claude_topic_core --test enumerate_test
cargo test -p claude_topic_core --test registry_test
```

On a real base, `clr topics` shows the live subset — every row it prints has a
conversation behind it:

```bash
clr topics
```

To see what the filter is doing, compare that against the raw traces of both
mechanisms:

```bash
ls -d ./-*/                                          # dir-mode candidates, unfiltered
cat "${CLR_TOPIC_REGISTRY_DIR:-$HOME/.clr/topics}"/*  # fork-mode names, unfiltered
```

Names present there but absent from `clr topics` are exactly the ones with zero
sessions — a directory nobody ran in, a registry entry whose file is gone, or a
`-`-prefixed scratch directory that was never a topic at all.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/enumerate.rs` | The merge, the sort, and the live filter |
| source | `src/registry.rs` | The side-channel index for fork-mode names |
| doc | [001_topic_identity.md](001_topic_identity.md) | The formulas that name what is being found |
| doc | [invariant/001_registry_non_authoritative.md](../invariant/001_registry_non_authoritative.md) | What the registry is not |
| doc | [invariant/002_mode_travels_with_name.md](../invariant/002_mode_travels_with_name.md) | Why a row is a pair |
| test | `tests/enumerate_test.rs` | Merge, sort order, live filter, per-mode session id |
| test | `tests/registry_test.rs` | Append-if-missing, newline refusal, per-base isolation |
