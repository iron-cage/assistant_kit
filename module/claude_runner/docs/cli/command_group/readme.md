# Command Groups

### Scope

- **Purpose**: Formalize sets of commands that share one implementing function and one parameter set, differing only in default values.
- **Responsibility**: Define command_group membership under a strict identity test — same handler function, same parameter set — distinct from the looser cross-command comparisons in `param_group/` and `parity/`.
- **In Scope**: Group membership, the Representation Absorption Test applied per candidate pair, shared-handler citations, default divergence (when any exists), and cross-references to commands/parameters/tests/user stories.
- **Out of Scope**: Individual parameter semantics (-> `../param/`), looser multi-command comparisons that don't share an identical parameter set (-> `../param_group/06_running_commands.md`, `../parity/`).

Every command in `command/` is evaluated against every other command using the Representation Absorption Test (see `01_run_ask.md` for the worked test) before a new command name is ever added — this is a mandatory design gate, not documentation-after-the-fact. A proposed new command that passes the test is a pre-configured alias of an existing command's handler, not a new command_group member requiring its own dispatch function.

Every command belongs to exactly one group — including a command with no sibling, which still opens its own Singleton Group and still receives a row below; no command is ever left out of **All Groups**. Before adding a new command, this same evaluation also asks whether an existing group's handler could absorb it via a new parameter instead of a new dispatch function — see the groups-to-commands ratio in the summary below. (Per this workspace-group's shared rulebook, CLI Documentation: Command Group Total Partition and Command Group Minimization.)

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 01_run_ask.md | Group spec: `run`/`ask`/`topic` — identical handler; `topic` diverges only in `--topic`'s default |

### All Groups (10 total)

| # | Group | Members | Shared Handler | Divergence |
|---|-------|---------|-----------------|------------|
| 1 | run / ask / topic | 3 | `dispatch_run()` (`src/cli/mod.rs:247`) | `topic` only — `--topic` default (see `01_run_ask.md`) |
| 2 | isolated | 1 | `dispatch_isolated()` (`src/cli/mod.rs:396`) | N/A — sole member |
| 3 | refresh | 1 | `dispatch_refresh()` (`src/cli/mod.rs:500`) | N/A — sole member |
| 4 | ps | 1 | `dispatch_ps()` (`src/cli/ps.rs:76`) | N/A — sole member |
| 5 | kill | 1 | `dispatch_kill()` (`src/cli/kill.rs:41`) | N/A — sole member |
| 6 | tools | 1 | `dispatch_tools()` (`src/cli/tools.rs:208`) | N/A — sole member |
| 7 | scope | 1 | `dispatch_scope()` (`src/cli/scope.rs:13`) | N/A — sole member |
| 8 | query | 1 | `dispatch_query()` (`src/cli/query.rs:90`) | N/A — sole member |
| 9 | help | 1 | N/A — intercepted pre-dispatch in `src/lib.rs`; no dispatch function | N/A — sole member |
| 10 | topics | 1 | `dispatch_topics()` (`src/cli/topics.rs:66`) | N/A — sole member |

**Total:** 10 groups for 12 commands (1 triple + 9 singletons). All 12 claude_runner commands were evaluated pairwise under the Representation Absorption Test — every one of the 11 dispatch functions (`dispatch_run`, `dispatch_ask`, `dispatch_topic`, `dispatch_topics`, `dispatch_isolated`, `dispatch_refresh`, `dispatch_ps`, `dispatch_kill`, `dispatch_tools`, `dispatch_scope`, `dispatch_query`; `help` is intercepted pre-dispatch in `src/lib.rs` and calls none) is invoked from exactly one call site in `src/lib.rs`'s top-level match, confirmed via a direct grep sweep for cross-calls among all 11 — the only delegations found are `dispatch_ask() -> dispatch_run()` and `dispatch_topic() -> dispatch_run()`. `run`/`ask`/`topic` is the only group sharing an identical dispatch function (with `topic` carrying one stated `--topic` default divergence); every other command forms a Singleton Group. See Evaluated, Not Qualifying below for the nearest misses considered and rejected.

### Evaluated, Not Qualifying

| Candidate Pair | Shared Implementation | Why Not a Command Group |
|-----------------|------------------------|---------------------------|
| `isolated` / `refresh` | `run_isolated_command()` (`src/cli/credential.rs`) | Different parameter surface — `refresh` drops `MESSAGE`, passthrough, `--dir`/`--add-dir`, `--file`, `--expect`/`--expect-strategy`, and all output-control flags rather than merely defaulting them differently. Fails the "exactly the same parameter set" bar even though both dispatch through the same lower-level function. See [`parity/002_isolated_refresh.md`](../parity/002_isolated_refresh.md) for the documented (looser) relationship. |
| `topics` / `topic` | None — `dispatch_topics()` (`src/cli/topics.rs:66`) and `dispatch_topic()` (`src/cli/topic.rs:94`) have zero cross-calls; both call `topic_path::topic_dir()` | Nearest miss for the newest command, and the only pair sharing the `<base>/-<name>` formula. Fails on both halves of the test. **Handler:** `topic` ends in `dispatch_run()` and a Claude subprocess; `topics` returns after a `read_dir` or a `PathBuf::join` and spawns nothing. **Parameter set:** `topic` accepts `run`'s full surface (~40 params, `MESSAGE` included); `topics` accepts 3 (`--path`, `--dir`, `--global`) and has its own hand-rolled loop rather than `parse_args()`. No default value of any `topic` parameter produces a directory listing — a listing is not a degenerate case of running something. `topic_path` is a shared internal path-computation module, exactly as `claude_storage_core::scope_for()` is for the `scope` / `run` pair below, and equally not a shared dispatch function. |
| `topics` / `ps` | None — `dispatch_topics()` (`src/cli/topics.rs:66`) and `dispatch_ps()` (`src/cli/ps.rs:76`) have zero cross-calls | Both are read-only enumerators printing an aligned plain-text table, but they enumerate different domains from different sources: `ps` scans the live process table for running `claude` processes, `topics` scans a base directory for `-<name>` entries whose sessions may all be long finished. Zero shared parameters — `ps` has `--mode`/`--wide`/`--pid`/`--columns`/`--inspect`, `topics` has `--path`/`--dir`/`--global`. Same output *shape*, disjoint parameter set and data source. |
| `topics` / `scope` | None — `dispatch_topics()` (`src/cli/topics.rs:66`) and `dispatch_scope()` (`src/cli/scope.rs:13`) have zero cross-calls; both use `claude_storage_core::scope_for()` | Closest by category (both are `--dir`-based, read-only, subprocess-free path inspectors) and they share one parameter, `--dir`. But `scope` answers "what are the 6 `CLAUDE_*` paths for *this one directory*" — a fixed 6-line projection of a single input; `topics` answers "what topic directories exist under this base" — a variable-length enumeration of *children*, with `--path` adding a name→path resolver `scope` has no analogue for. `topics` uses `scope_for()` for one field (`claude_session_dir`) as one step in counting sessions per topic, the same 1-of-6 incidental use that keeps `run` out of `scope`'s orbit below. |
| `run` / `query` | None — `dispatch_run()` (`src/cli/mod.rs:247`) and `dispatch_query()` (`src/cli/query.rs:90`) have zero cross-calls | `command/10_query.md`'s Related Commands table describes `run` as sharing "the backgrounded-by-default session model query's start form mirrors" — a behavioral similarity (both return immediately with a PID rather than blocking), not implementation sharing. Parameter sets diverge entirely beyond `--dir`: `query` adds a `<PID> <METHOD> [ARGS...]` dispatch form (25 control methods) with no `run` equivalent; `run`'s ~30 Claude-native/runner-control flags have no `query` equivalent. `query.md` itself already disclaims shared parsing for `--dir` ("not routed through the Runner Control group's shared argument parser"). See [`command/10_query.md`](../command/10_query.md#referenced-command-group). |
| `ps` / `tools` | None — `dispatch_ps()` (`src/cli/ps.rs:76`) and `dispatch_tools()` (`src/cli/tools.rs:208`) have zero cross-calls | Both register the literal same `--columns` (`param/059_columns.md`) and `--inspect` (`param/069_inspect.md`) parameter docs and render similar plain-style output — but that's 2 of 5 params on each side. `ps` also has `--mode`/`--wide`/`--pid` with no `tools` equivalent; `tools` also has `--name`/`--category`/`--value` with no `ps` equivalent. Shared parameter *names* on a minority of each command's surface, not a shared parameter *set*. |
| `scope` / `run`+`ask` | None — `dispatch_scope()` (`src/cli/scope.rs:13`) and `dispatch_run()`/`dispatch_ask()` have zero cross-calls | Both call `claude_storage_core::scope_for()` — an external library utility (from the `claude_storage_core` crate), not a claude_runner dispatch function. `scope.rs:56` calls it for its entire purpose (prints all 6 returned fields); `builder.rs`'s `build_claude_command()` calls it unconditionally from `--from` (which defaults to CWD) to compute `session_from_dir` — `--session-dir` is never consulted here (BUG-493 dropped the `cli.session_dir.is_none()` gate that used to make this call conditional) — using 1 of 6 fields as one step inside `run`'s much larger subprocess-argument builder. `scope`'s 1-param surface (`--dir`) is already documented as a `Running Commands` param_group subset (see [`param_group/06_running_commands.md`](../param_group/06_running_commands.md)) — a parameter-level relationship, not a shared-dispatch one. See [`command/09_scope.md`](../command/09_scope.md#referenced-command-group). |

### Navigation

- [run / ask / topic](01_run_ask.md)
