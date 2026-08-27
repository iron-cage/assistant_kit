# CLI Parameter: --global

Redirects `--topic`'s base directory from the current working directory to the **global
topic home**, so a topic is addressable by name from anywhere instead of only from the
project it was created in.

- **Type:** boolean flag
- **Default:** `false` (topic directories are created under the current working directory)
- **Alias:** `-g`
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md), [`topic`](../command/11_topic.md), [`topics`](../command/12_topics.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"global"`

```sh
clr --global --topic notes "Jot this down"   # effective dir = $CLR_TOPIC_HOME/-notes
clr topic --global "Investigate the leak"    # auto-named, under the global home
clr topics --global                          # list global topics
clr topics --global --path notes             # resolve one global topic's path
```

**Base precedence** (highest first), implemented once in `claude_topic_core::identity::topic_base()`
and shared by every consumer:

| # | Condition | Base |
|---|-----------|------|
| 1 | `--dir <PATH>` given | `<PATH>` — an explicit path always outranks a named default, so `--dir` wins even alongside `--global` |
| 2 | `--global` given | The global topic home (below) |
| 3 | neither | Current working directory |

**Global topic home:** `$CLR_TOPIC_HOME` when set to a non-empty value, otherwise
`<system temp dir>/clr-topic` (`/tmp/clr-topic` on a typical Linux host). On most systems
the temp dir is cleared on reboot — set `CLR_TOPIC_HOME` explicitly for topics that must
outlive one.

**Inert without a topic.** `--global` only redirects `--topic`'s base. With no topic
directory to place (`--topic` absent, `.`, or `""`), there is no base to redirect and
`--dir`/CWD stands unchanged — `clr --global "task"` is byte-identical to `clr "task"`.

**Deterministic name-to-path.** Because the base is a function of the environment rather
than of the current directory, a global topic's path is recoverable from its name alone in
any later shell: `clr topics --global --path <NAME>` prints it without touching the disk.

**Env var:** `CLR_GLOBAL` — boolean; applied when `--global` is absent from the CLI.
`CLR_GLOBAL=1 clr --topic notes "task"` is equivalent to `clr --global --topic notes "task"`.
Note the two distinct variables: `CLR_GLOBAL` turns the flag on, `CLR_TOPIC_HOME` chooses
where the global base is.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| boolean | Primitive | bool | Presence-only flag — takes no value; `--global true` would parse `true` as the message |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 17 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | `false` | Redirects an explicit `--topic`'s base |
| 5 | [`ask`](../command/05_ask.md) | `false` | Same as `run` |
| 11 | [`topic`](../command/11_topic.md) | `false` | Also redirects where the auto-generated slug probes for a free name |
| 12 | [`topics`](../command/12_topics.md) | `false` | Selects which base is listed / resolved against |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 30 | [030_topic_creation.md](../user_story/030_topic_creation.md) | Developer |
| 31 | [031_topic_discovery.md](../user_story/031_topic_discovery.md) | Developer |
