# Feature: Topic Identity

### Scope

- **Purpose**: Define what a `--topic` name resolves to — a base directory, one of two mechanisms, and a session — and which mechanism answers when the caller does not say.
- **In Scope**: `TopicMode`, `effective_topic_mode`, `topic_home`, `topic_base`, `topic_dir`, `topic_name_of`, `fork_session_file`.
- **Out of Scope**: Actually creating the session — forking, transplanting, resuming (→ [`claude_runner`](../../../claude_runner/docs/cli/command/readme.md)); the `UUIDv5` rule and the storage layout it lands in (→ [`claude_storage_core`](../../../claude_storage_core/docs/readme.md)).

### Why This Exists

Two mechanisms answer to the word "topic", and they have almost nothing in common.

**Fork mode** creates no directory at all. The topic is a session file named
`UUIDv5( canonical base path, topic name )`, living in the base directory's *own*
session storage, created by forking the base's most recent session. Because the
working directory never changes, the prompt-cache prefix stays byte-identical and
the fork reuses the base session's cache — measured at roughly 5% of a cold prime,
against roughly 77% for a directory change.

**Dir mode** is the legacy mechanism: a `<base>/-<name>` working directory, plus a
physical transplant of a session file into that directory's storage. It is what a
cross-directory source or a shared global topic needs, and it is what every topic
created before fork mode existed already is.

Both are real, both are supported, and a name can be held by both at once. Which
one a bare `--topic review` reaches is therefore a decision, not a detail — and
it is the decision this feature makes.

### Mode Precedence

`effective_topic_mode` applies five rules, highest first. The first that matches wins.

| # | Condition | Mode | Why |
|---|-----------|------|-----|
| 1 | An explicit `--topic-mode` / `CLR_TOPIC_MODE` / json `topic-mode` | as given | The caller said so; nothing outranks that |
| 2 | `--global` | Dir | A global topic is shared across arbitrary callers' directories, so fork mode's same-directory cache premise never holds |
| 3 | A non-empty `--from` | Dir | An explicit cross-directory source needs the transplant machinery, and a cross-directory prefix cannot cache-hit anyway |
| 4 | `<base>/-<name>` already exists | Dir | A topic created by the legacy mechanism keeps its accumulated history; a fork silently starting a parallel same-name session would orphan it |
| 5 | otherwise | Fork | The default for every new topic |

Rule 4 is the one with a consequence worth stating plainly: **once a name exists in
dir mode, a bare `--topic <name>` can never reach that name's fork-mode twin.** Rule
1 is the only way back, and that is why the mode has to travel with the name
wherever a topic is addressed — see [invariant/002](../invariant/002_mode_travels_with_name.md).

### Base Resolution

The base is the directory topics belong to, and it is *not* always the current one.

| # | Condition | Base |
|---|-----------|------|
| 1 | `--dir <PATH>` | `PATH` — an explicit path beats a named default, so `--dir` wins even alongside `--global` |
| 2 | `--global` | `$CLR_TOPIC_HOME`, or `<system temp dir>/clr-topic` |
| 3 | otherwise | The current working directory |

`CLR_TOPIC_HOME` is used verbatim — nothing is appended to it. On most systems the
system temp dir is cleared on reboot, so a global topic that must outlive one needs
`CLR_TOPIC_HOME` set explicitly.

### Naming

`topic_dir( base, name )` is `<base>/-<name>`, and `topic_name_of` is its inverse.
The hyphen is unconditional: it is what makes a topic directory recognisable when
scanning a base, and it is also the workspace-wide marker for generated and ignored
directories — a collision this crate does not resolve by name, and handles instead
in [feature/002](002_topic_enumeration.md).

A bare `-` is not a topic named `""`; `topic_name_of` returns `None` for it.

### Purity

Everything here is pure path arithmetic, with two named exceptions:

- `topic_home` reads `CLR_TOPIC_HOME`, and `topic_base` may read the current directory.
- `effective_topic_mode` performs exactly one filesystem probe, for rule 4.

That exception is not incidental. Path resolution answers "where would this go?",
which must work whether or not anything exists. Mode selection answers "which
mechanism does this name already belong to?", which is *defined* in terms of what
exists, and cannot be computed without looking.

### Verification

```bash
cd module/claude_topic_core && ./verb/test
```

Or the single test binary, in-container:

```bash
cargo test -p claude_topic_core --test identity_test
```

`tests/identity_test.rs` covers all five precedence rules. The one that matters most
is tid13 — an explicit `fork` must beat an existing directory. An implementation
that checks rule 4 before rule 1 passes every other case in the file.

To see what a name resolves to without creating anything — both mechanisms, since
each has its own answer:

```bash
clr topics --path review   # the dir-mode working directory: <base>/-review
clr topics --file review   # the fork-mode session file: <base storage>/<uuid>.jsonl
```

Both print a path whether or not it exists, because both are computed rather than
looked up.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/identity.rs` | `TopicMode`, precedence, path formulas |
| doc | [002_topic_enumeration.md](002_topic_enumeration.md) | Finding what these formulas name |
| doc | [invariant/002_mode_travels_with_name.md](../invariant/002_mode_travels_with_name.md) | Why the mode is part of the address |
| doc | [api/001_topic_surface.md](../api/001_topic_surface.md) | Full signature contract |
| doc | [`claude_storage_core` topic session](../../../claude_storage_core/docs/readme.md) | The `UUIDv5` rule and storage layout |
| test | `tests/identity_test.rs` | Precedence, base resolution, name round-tripping |
