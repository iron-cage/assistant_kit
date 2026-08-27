# CLI Parameter: --topic

Names an isolated conversation for the effective working directory. A NEW topic
defaults to a same-directory session **fork** (no topic directory is created); a
topic with an existing `-<name>` directory, a `--from` source, or `--global` keeps
the legacy **dir** mechanism, where `/-<name>` is appended to the base to form the
execution directory. Default `.` is the identity value — no topic at all.

- **Type:** string (single name component; no `/` separators; `.` or `""` = identity)
- **Default:** `.` (identity — no topic)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md), [`topic`](../command/11_topic.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)
- **JSON Key:** `"topic"`
- **Mode override:** [`--topic-mode`](088_topic_mode.md) (`fork`/`dir`)

```sh
clr "Fix bug"                           # no topic (default: --topic .)
clr --topic build "Fix bug"            # NEW topic: same-dir session fork (fork mode)
clr --dir /project --topic debug "x"  # /project/-debug exists? dir mode : fork mode
clr --topic . "Fix bug"               # explicit identity — same as default
```

**Mode selection (fork vs dir):** decided per invocation by
`claude_topic_core::effective_topic_mode`, precedence highest first:

1. Explicit [`--topic-mode`](088_topic_mode.md) / `CLR_TOPIC_MODE` / json `"topic-mode"`.
2. `--global` → dir — a global topic is shared across callers' working directories, so
   fork mode's same-directory cache premise never holds.
3. Non-empty `--from` → dir — an explicit cross-directory source needs the transplant
   machinery.
4. Existing `<base>/-<name>` directory → dir — a legacy topic keeps its accumulated
   directory-based history; fork mode will not orphan it with a parallel same-name session.
5. Otherwise → fork — the default for every new topic.

**Fork mode (default for new topics):** the subprocess stays in the base directory; the
topic lives as a deterministically-named session file
`{storage of base}/{UUIDv5( canonical base, name )}.jsonl` inside the base's own storage.
First use forks the base's most recent session
(`--resume <source> --fork-session --session-id <topic-uuid>`; with no source:
`--session-id <topic-uuid>` alone); every repeat use resumes it (`--resume <topic-uuid>`).
Staying in the base keeps the prompt-cache prefix byte-identical, so a fork re-reads the
base session's cache (~5% of a cold prime) instead of re-priming the whole history
(~77% after a directory change). The name → session-file mapping is resolvable via
[`topics --file NAME`](../command/12_topics.md); names are recorded (append-only,
warn-never-fatal) in the topics registry (`CLR_TOPIC_REGISTRY_DIR` > `~/.clr/topics/`)
so the listing can recover them from the one-way UUID. Dry-run previews the plan as
`# topic-fork: ...` / `# topic-resume: ...` and writes nothing.

**Dir mode (legacy):** `/-<name>` is appended to the base directory (`--dir` value or
cwd) and created automatically (`create_dir_all`) before subprocess spawn — no manual
`mkdir` needed. Claude Code session state is keyed by working directory, so each topic
directory holds an independent conversation history; first use physically transplants
the source session into it (see [`--from`](076_from.md)). In dry-run mode, directory
creation is suppressed so `--dry-run` remains side-effect-free.

**Identity values:** Both `.` (explicit) and `""` (empty string) are treated as identity —
no fork plan, no `/-` suffix, no directory created.

**Validation:** Values containing `/` are rejected at parse time (`--topic must be a
single directory name component (no '/' separators)`). Use `--dir` for base directory
scoping; `--topic` is the final name only.

**Session isolation:** both modes give `--topic build` and `--topic debug` within the
same `--dir` independent conversation histories — dir mode by working directory, fork
mode by deterministic session id. This is the mechanism wplan uses to isolate per-topic
workspaces: `dream .claude topic::build` resolves to `clr --dir /project/-build "..."`.

**Note:** The `-` prefix in a dir-mode topic directory name (`/-build`) follows the
project transient-directory convention — directories beginning with `-` are git-excluded
by `.gitignore` patterns.

**Env var:** `CLR_TOPIC` — string; applied when `--topic` is absent from the CLI
and `CLR_TOPIC` is non-empty. `CLR_TOPIC=build clr "task"` is equivalent to
`clr --topic build "task"`.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| string | Primitive | &str | `.` or `""` (identity) or valid single name component (no `/`; validated at parse time) |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 16 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | `.` (identity) | — |
| 5 | [`ask`](../command/05_ask.md) | `.` (identity) | — |
| 11 | [`topic`](../command/11_topic.md) | auto-generated slug from `MESSAGE` | Only command diverging from `.`; explicit `--topic NAME` overrides the auto-generated slug |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 22 | [022_session_isolation_topic.md](../user_story/022_session_isolation_topic.md) | Developer |
| 30 | [030_topic_creation.md](../user_story/030_topic_creation.md) | Developer |
