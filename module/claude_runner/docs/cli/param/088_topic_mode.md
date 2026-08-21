# CLI Parameter: --topic-mode

Forces which of the two topic mechanisms `--topic` uses — `fork` (same-directory
session fork, the default for new topics) or `dir` (legacy `-<name>` working
directory + session transplant) — overriding the automatic selection rules.

- **Type:** enum — `fork` | `dir`
- **Default:** absent (automatic selection — see [`--topic`](028_topic.md) § Mode selection)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md), [`topic`](../command/11_topic.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"topic-mode"`

```sh
clr --topic x "task"                       # auto: new topic → fork mode
clr --topic-mode dir --topic x "task"     # force legacy dir mode for a new topic
clr --topic-mode fork --topic x "task"    # force fork even when a -x dir exists
clr --topic-mode fork --dry-run --topic x # preview: '# topic-fork: ...' line, no side effects
```

**Precedence:** CLI `--topic-mode` > env `CLR_TOPIC_MODE` > json `"topic-mode"` >
automatic selection. An explicit mode from any of these three sources beats every
automatic rule — including the existing-directory heuristic, so
`--topic-mode fork --topic x` forks even when `<base>/-x` already exists (the
directory topic is left untouched; a parallel fork-mode session with the same name
begins).

**Validation:** any value other than `fork` or `dir` is rejected at parse time
(`invalid topic mode: <VALUE>` / `Expected: fork or dir`).

**Contradictions exit 1 rather than guess.** The automatic rules select dir mode for
`--global` and `--from` because fork mode's same-directory cache-identity premise cannot
hold for them; forcing past that is a contradiction:

| Combination | Error |
|-------------|-------|
| `--topic-mode fork --global` | `--topic-mode fork cannot be combined with --global` |
| `--topic-mode fork --from <SRC>` (non-empty) | `--topic-mode fork cannot be combined with --from` |

**Inert without a topic.** Like [`--global`](087_global.md), `--topic-mode` only
configures `--topic`'s mechanism. With no topic (`--topic` absent, `.`, or `""`) there
is no mode to select and the value is ignored.

**Why force `dir`:** compatibility with tooling that expects a per-topic working
directory on disk (e.g. wplan's `/-<name>` resolution), or to give a topic its own
directory-scoped file state. **Why force `fork`:** reclaim cache reuse for a name whose
`-<name>` directory exists but is no longer wanted as a directory topic.

**Env var:** `CLR_TOPIC_MODE` — same `fork`/`dir` values; applied when `--topic-mode`
is absent from the CLI. Invalid env values are silently ignored (env fallbacks never
abort the run), unlike the CLI flag's loud parse error.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| enum | `TopicMode` (`src/cli/topic_path.rs`) | &str | Exactly `fork` or `dir`; anything else is a parse error |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 54 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | absent (auto) | Forces the mechanism for an explicit `--topic` |
| 5 | [`ask`](../command/05_ask.md) | absent (auto) | Same as `run` |
| 11 | [`topic`](../command/11_topic.md) | absent (auto) | Also governs the auto-named slug's mechanism |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 22 | [022_session_isolation_topic.md](../user_story/022_session_isolation_topic.md) | Developer |
| 30 | [030_topic_creation.md](../user_story/030_topic_creation.md) | Developer |
