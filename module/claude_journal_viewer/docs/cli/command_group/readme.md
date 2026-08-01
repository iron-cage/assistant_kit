# Command Groups

### Scope

- **Purpose**: Formalize sets of commands that share one implementing function and one parameter set, differing only in default values.
- **Responsibility**: Define command_group membership under a strict identity test — same handler function, same parameter set — distinct from the looser cross-command comparisons in `param_group/`.
- **In Scope**: Group membership, the Representation Absorption Test applied per candidate pair, shared-handler citations, default divergence (when any exists), and cross-references to commands/parameters/tests/user stories.
- **Out of Scope**: Individual parameter semantics (-> `../param/`), looser multi-command comparisons that don't share an identical parameter set (-> `../param_group/`).

Every command in `command/` is evaluated against every other command using the Representation Absorption Test before a new command name is ever added — this is a mandatory design gate, not documentation-after-the-fact.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| *(none)* | No qualifying command_group exists yet — see Evaluated, Not Qualifying below |

### All Groups (0 total)

| # | Group | Members | Shared Handler | Divergence |
|---|-------|---------|-----------------|------------|

**Total:** 0 groups. All 8 documented `clj` commands (`.list`, `.tail`, `.stats`, `.search`, `.serve`, `.prune`, `.status`, `.export`) were evaluated pairwise under the Representation Absorption Test against `src/cli_main.rs`. Every `cmd_*` dispatch function calls a distinct implementing function — `list_output()`, `stats_output()`, `search_output()`, `prune_output()`, `status_output()`, `export_output()` (all in `src/output.rs`), plus `cmd_tail()` and `cmd_serve()`'s own inline logic (`src/cli_main.rs`). No two of the 8 documented commands share a dispatch/handler function, so none can qualify regardless of parameter overlap.

### Evaluated, Not Qualifying

| Candidate Pair | Shared Implementation | Why Not a Command Group |
|-----------------|------------------------|---------------------------|
| `clj .list` / `ast .journal.list` | `output::list_output()` (`src/output.rs`) — called by both `cmd_list()` in `src/cli_main.rs:89` and `list_routine()` in `src/routines.rs:51` | `list_routine` is not a documented command under `command/` — it is part of the `routines` feature (`Cargo.toml`: `routines = ["dep:unilang", "unilang/enabled"]`), an optional, non-default super-app integration surface with no doc instance of its own. Out of this entity's scope regardless of the shared handler. |
| `clj .stats` / `ast .journal.stats` | `output::stats_output()` — `cmd_stats()` (`src/cli_main.rs:115`) and `stats_routine()` (`src/routines.rs:68`) | Same reason as above — `stats_routine` is not a documented `command/` entry. |
| `clj .search` / `ast .journal.search` | `output::search_output()` — `cmd_search()` (`src/cli_main.rs:125`) and `search_routine()` (`src/routines.rs:85`) | Same reason as above. |
| `clj .status` / `ast .journal.status` | `output::status_output()` — `cmd_status()` (`src/cli_main.rs:145`) and `status_routine()` (`src/routines.rs:102`) | Same reason as above. |
| `clj .export` / `ast .journal.export` | `output::export_output()` — `cmd_export()` (`src/cli_main.rs:151`) and `export_routine()` (`src/routines.rs:117`) | Same reason as above. |
| `clj .tail` / `ast .journal.tail` | None — `tail_routine()` (`src/routines.rs:153`) is a stub returning a fixed guidance string; it never calls `build_filter()` or `JournalReader::tail()` | Fails on two independent grounds: not a documented `command/` entry, and (even setting that aside) genuinely different behavior — `cmd_tail` performs the real tail, `tail_routine` does not. |
| `clj .serve` / `ast .journal.serve` | None — `serve_routine()` (`src/routines.rs:171`) is a stub returning a fixed guidance string; it never binds an HTTP server | Same two-fold rejection as `.tail` above — undocumented surface plus genuinely different behavior. |

No pre-existing `param_group/`-style or `parity/`-style file in this crate asserts a shared-handler claim between any two commands — unlike claude_runner's `param_group/06_running_commands.md`, there is nothing here for this entity to formalize or cross-reference; this readme is the first place any of these relationships are documented at all.

### Navigation

*(none — no qualifying groups; see Evaluated, Not Qualifying above)*
