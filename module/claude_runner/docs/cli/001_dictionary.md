# Dictionary

### Scope

- **Purpose**: Provide domain vocabulary for clr CLI concepts, modes, and architecture.
- **Responsibility**: Define canonical terms used throughout the CLI documentation.
- **In Scope**: Command terms, mode terms, architecture terms.
- **Out of Scope**: Type definitions with full specs (→ `type/`), parameter usage (→ `param/`).

### Commands

| Term | Definition |
|------|------------|
| run | Default command that builds and executes a `claude` subprocess with the given flags |
| ask | Semantic alias for `run` with identical parameters and defaults; no behavioral differences |
| isolated | Subcommand that runs `claude` in a credential-isolated temporary HOME; requires `--creds` |
| refresh | Subcommand that refreshes OAuth credentials via `run_isolated()` with `["--print", "."]`; requires `--creds`; no task executed |
| ps | List running Claude Code sessions with process metrics; supports `--mode`, `--columns`, `--wide`, `--pid`, `--inspect` |
| kill | Terminate a running Claude Code session by PID via SIGTERM; canonical form `clr kill <PID>` |
| tools | List all Claude Code built-in tools available to the subprocess; canonical form `clr tools` |
| scope | Print all 6 `CLAUDE_*` path variables for a directory; canonical form `clr scope [--dir <DIR>]` |
| query | Start or dispatch against a persistent PID-addressed control session; canonical form `clr query "<message>"` or `clr query <pid> <method>` |
| topic | Create a topic session (fork mode for new names, dir mode for existing `-NAME` directories), or continue one if it already exists; auto-names via slug generation when `--topic` is omitted; canonical form `clr topic "<prompt>"` |
| topics | List the topics under a base (both modes), or resolve one topic name to its dir-mode directory (`--path`) or fork-mode session file (`--file`); read-only, spawns nothing; canonical form `clr topics`, `clr topics --path <NAME>`, or `clr topics --file <NAME>` |
| daemon | Manage the single session daemon — the long-lived process hosting interactive sessions on terminals of their own; canonical forms `clr daemon status` (also the bare `clr daemon`), `clr daemon start`, `clr daemon stop`, `clr daemon log` |
| chat | Send one prompt to a hosted interactive session and print the answer, leaving the session alive to continue the conversation; reuses the session in the working directory, or starts the daemon and a session when there is none; canonical form `clr chat "<message>"` |
| sessions | List the sessions the daemon is hosting — conversation id, PID, turn state, and directory; a pure query that starts no daemon, unlike `chat`; canonical forms `clr sessions` and `clr sessions --json` |
| help | Display usage information and exit; canonical form `clr help`; `--help`/`-h` are parameter aliases |

<!-- BUG-480 — fixed: the three per-surface "active" senses and "slot occupancy" are defined in § Architecture below -->
### Modes

| Term | Definition |
|------|------------|
| interactive mode | Default TTY passthrough mode; stdin/stdout connected directly to the claude subprocess; continues previous session automatically |
| print mode | Non-interactive capture mode (`-p`/`--print`); stdout collected and printed for programmatic use; continues previous session automatically |
| dry-run | Preview mode (`--dry-run`); prints the assembled command without executing it; output shows `-c` when a prior session exists for the effective working directory |
| new session | Invocation with `--new-session`; starts a fresh Claude conversation with no prior context (omits the default `-c`) |
| ultrathink suffix | Text `"\n\nultrathink"` appended after every message before it is sent to the claude subprocess; activates Claude's extended thinking mode; default-on, suppressed with `--no-ultrathink` |
| credential-isolated mode | Invocation via `clr isolated`; subprocess runs with a temporary HOME containing only the provided credentials file; the caller's real HOME, settings, and conversation history are invisible to the subprocess |
| fence stripping | Post-processing of captured stdout by `--strip-fences`; removes the first and last `` ``` `` lines (with optional language tag); content between the fences is emitted unchanged; no-op if no fence pair is found |
| standalone mode | Default subprocess behavior: `CLAUDECODE` env var is removed before spawn so the subprocess behaves as a first-class Claude Code process, not a nested agent; opt out with `--keep-claudecode` |
| nested-agent mode | Subprocess behavior when `CLAUDECODE=1` is inherited from the parent; alters permission handling, output format, and tool availability; active when `--keep-claudecode` is set |
| credential refresh mode | Invocation via `clr refresh`; subprocess runs with `["--print", "."]` in a temporary HOME to trigger OAuth token refresh at startup; no user task is executed; default timeout 45 seconds |

### Architecture

| Term | Definition |
|------|------------|
| Claude-native flag | A flag forwarded to the claude subprocess (e.g., `--model`, `--verbose`) |
| runner-specific flag | A flag consumed by the runner itself, not forwarded to claude (e.g., `--dry-run`, `--quiet`, `--new-session`) |
| session continuation (automatic) | Default behavior: `-c` is passed to the claude subprocess when a prior session exists for the effective working directory and `--new-session` is not given; resumes the most recent conversation |
| ClaudeCommand | Builder pattern from `claude_runner_core` that assembles the subprocess invocation |
| session directory | Filesystem location where Claude Code persists conversation state; `clr` continues the session stored here by default |
| `--` separator | Double-dash token; everything after it becomes positional (part of the message) |
| last-wins | When a flag appears multiple times, the last occurrence takes effect |
| temp HOME | Temporary directory created by `clr isolated` containing only `.claude/.credentials.json`; set as `HOME` for the subprocess; deleted unconditionally on exit regardless of timeout or error |
| active (gate census) | The `N` in the gate-wait line's `active=N/M`: count of live print-mode sessions observed via `{pid}.json` telemetry files — the census conjunct of gate admission; says nothing about how many slots are held (see slot occupancy) |
| active (ps summary) | The `N` in `clr ps`'s `Active Sessions · N running` caption: all live sessions across every mode (interactive, print, query), not only print-mode |
| active (⚡ flag) | Per-row `clr ps` flag: the session's process consumed ≥ 3 CPU ticks in a 1-second sample window — CPU activity, unrelated to either census sense |
| slot occupancy | Count of gate slot files (`slot_N.json`) whose recorded owner is alive, out of `max-sessions` — the slot-CAS conjunct of gate admission; surfaced as `slots=H/M` on slot-side denial diagnostics and `slots=H/M held` on gate-exhaustion messages (BUG-480) |
| topic (concept) | A named, isolated line of work with its own conversation. The canonical term for this concept — never "subdir". Realized by one of two mechanisms, fork mode (default for new topics) or dir mode (legacy). For the two commands of the same name see [Commands](#commands) above |
| topic name | The bare name identifying a topic (`auth-refactor`). A single name component — a value containing `/` is rejected. Carried by `--topic` / `CLR_TOPIC` / JSON `"topic"`. The same name can exist once per mode |
| fork mode | Topic mechanism where the subprocess stays in the base directory and the topic lives as a deterministically-named session file — `UUIDv5( canonical base, name )` — in the base's own storage, created by forking the base's most recent session (`--fork-session`). Preserves the base session's prompt cache. Default for new topics; forced via `--topic-mode fork` |
| dir mode | Legacy topic mechanism: a `-NAME` working directory under the base plus a physical session-file transplant. Selected automatically for existing topic directories, `--from`, and `--global`; forced via `--topic-mode dir` |
| topic directory | The on-disk directory backing a dir-mode topic: the topic name prefixed with `-` (topic name `auth-refactor` → directory `-auth-refactor`), placed under the base. The `-` prefix is what makes it a topic directory and what `clr topics` recognizes when listing; a plain directory of the same name is not a topic. Fork-mode topics have no topic directory |
| topic session file | The session file embodying a fork-mode topic: `<storage of base>/<UUIDv5( canonical base, name )>.jsonl`. Resolved by `clr topics --file <NAME>`, byte-identical to `claude_storage .session.path path::<base> topic::<NAME>` |
| topics registry | Side-channel index of fork-topic names (`CLR_TOPIC_REGISTRY_DIR` > `~/.clr/topics/`; one file per base, one name per line) enabling `clr topics` to list fork topics — the UUIDv5 identity is one-way, so names cannot be recovered from session files. Convenience index, never an authority; append-if-missing, warn-never-fatal |
| topic session | A Claude session belonging to a topic — inside the topic directory (dir mode) or the topic session file itself (fork mode). Distinct from the topic's existence: a dir-mode topic can exist with zero sessions (a `SESSIONS` count of `0` in `clr topics` output), and a fork row shows `0` when its registry entry outlives a deleted session file |
| base | The directory that topic directories are created and listed under. Resolved by precedence, highest first: `--dir <PATH>`, then `--global`, then the current working directory. Single source of truth: `claude_topic_core::topic_base()` |
| global topic home | The one particular base that `--global` selects: `$CLR_TOPIC_HOME` if set, else `<system temp dir>/clr-topic`. `CLR_GLOBAL` turns the flag on; `CLR_TOPIC_HOME` chooses where the home is — two distinct variables |
