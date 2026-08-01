# Command Groups

### Scope

- **Purpose**: Partition every documented command into exactly one command_group — a set of commands sharing one implementing function and one parameter set, differing only in default values (or a singleton, when no sibling qualifies).
- **Responsibility**: Define command_group membership under a strict identity test — same handler function, same parameter set — distinct from the looser cross-command comparisons in `param_group/`. Every command belongs to exactly one group, with no exceptions — a command with no qualifying sibling still opens its own Singleton Group and still receives a row in **All Groups**; it is never omitted.
- **In Scope**: Group membership (including mandatory singletons), the Representation Absorption Test applied per candidate pair, shared-handler citations, default divergence (when any exists), the groups-to-commands minimization ratio, and cross-references to commands/parameters/tests/user stories.
- **Out of Scope**: Individual parameter semantics (-> `../param/`), looser multi-command comparisons that don't share an identical parameter set (-> `../param_group/`).

Every command in `command/` is evaluated against every other command using the Representation Absorption Test before a new command name is ever added — this is a mandatory design gate, not documentation-after-the-fact. Every command belongs to exactly one group — including a command with no sibling, which still opens its own Singleton Group and still receives a row below; no command is ever left out of **All Groups**. Before adding a new command, this same evaluation also asks whether an existing group's handler could absorb it via a new parameter instead of a new dispatch function. (Per this workspace-group's shared rulebook, CLI Documentation: Command Group Total Partition and Command Group Minimization.)

### Responsibility Table

| File | Responsibility |
|------|----------------|
| *(none)* | No qualifying multi-member command_group exists — see All Groups below for the 8 mandatory singletons |

### All Groups (8 total)

| # | Group | Members | Shared Handler | Divergence |
|---|-------|---------|-----------------|------------|
| 1 | `.list` | 1 | `cmd_list()` (`src/cli_main.rs:89`) | N/A — sole member |
| 2 | `.tail` | 1 | `cmd_tail()` (`src/cli_main.rs:99`) | N/A — sole member |
| 3 | `.stats` | 1 | `cmd_stats()` (`src/cli_main.rs:115`) | N/A — sole member |
| 4 | `.search` | 1 | `cmd_search()` (`src/cli_main.rs:125`) | N/A — sole member |
| 5 | `.prune` | 1 | `cmd_prune()` (`src/cli_main.rs:135`) | N/A — sole member |
| 6 | `.status` | 1 | `cmd_status()` (`src/cli_main.rs:145`) | N/A — sole member |
| 7 | `.export` | 1 | `cmd_export()` (`src/cli_main.rs:151`) | N/A — sole member |
| 8 | `.serve` | 1 | `cmd_serve()` (`src/cli_main.rs:161`) | N/A — sole member |

**Total:** 8 groups for 8 commands (8 singletons) — a 1:1 groups-to-commands ratio, the maximum possible under the Total Partition requirement, reflecting that this sweep found zero multi-member groups. All 8 documented `clj` commands (`.list`, `.tail`, `.stats`, `.search`, `.serve`, `.prune`, `.status`, `.export`) were evaluated pairwise under the Representation Absorption Test against `src/cli_main.rs`; each opens its own Singleton Group (§ Vocabulary : Singleton Group) since none passes the test against any sibling.

**Verification method:** every `cmd_*` dispatch function defined in `src/cli_main.rs` — `cmd_list()` (line 89), `cmd_tail()` (line 99), `cmd_stats()` (line 115), `cmd_search()` (line 125), `cmd_prune()` (line 135), `cmd_status()` (line 145), `cmd_export()` (line 151), `cmd_serve()` (line 161) — was checked with a full pairwise grep sweep of `src/` for every call site of each function name (excluding its own `fn` definition line):

```bash
for fn in cmd_list cmd_tail cmd_stats cmd_search cmd_serve cmd_prune cmd_status cmd_export; do
  grep -rn "$fn(" src/ tests/ | grep -v "fn $fn"
done
```

The sweep result: each `cmd_*` function has **exactly one call site**, the `match command { ... }` dispatch block in `main()` (`src/cli_main.rs:262-279`) — no `cmd_*` function calls another, and no `cmd_*` function is called from anywhere outside `main()`. This is the fact backing "no two of the 8 documented commands share a dispatch/handler function," and therefore backing all 8 rows above being Singleton Groups — it is a swept, grepped result, not informal reasoning.

One layer deeper, each `cmd_*` function calls a distinct implementing function — `list_output()` (`src/output.rs:180`), `stats_output()` (`src/output.rs:219`), `search_output()` (`src/output.rs:279`), `prune_output()` (`src/output.rs:340`), `status_output()` (`src/output.rs:318`), `export_output()` (`src/output.rs:461`) — each swept the same way and confirmed to have exactly 2 call sites crate-wide (its own `cmd_*` caller in `src/cli_main.rs`, plus its `*_routine()` caller in `src/routines.rs` under the optional `routines` feature — see the `ast .journal.*` rows below), never a second `cmd_*` caller. `cmd_tail()` and `cmd_serve()` use their own inline logic in `src/cli_main.rs` rather than an `output::` function. No two of the 8 documented commands share a dispatch/handler function at either layer, so none can qualify for a multi-member group regardless of parameter overlap — each therefore opens its own Singleton Group above.

### Evaluated, Not Qualifying

| Candidate Pair | Shared Implementation | Why Not a Command Group |
|-----------------|------------------------|---------------------------|
| `clj .list` / `ast .journal.list` | `output::list_output()` (`src/output.rs:180`) — called by both `cmd_list()` in `src/cli_main.rs:89` and `list_routine()` in `src/routines.rs:51` | `list_routine` is not a documented command under `command/` — it is part of the `routines` feature (`Cargo.toml`: `routines = ["dep:unilang", "unilang/enabled"]`), an optional, non-default super-app integration surface with no doc instance of its own. Out of this entity's scope regardless of the shared handler. |
| `clj .stats` / `ast .journal.stats` | `output::stats_output()` (`src/output.rs:219`) — `cmd_stats()` (`src/cli_main.rs:115`) and `stats_routine()` (`src/routines.rs:68`) | Same reason as above — `stats_routine` is not a documented `command/` entry. |
| `clj .search` / `ast .journal.search` | `output::search_output()` (`src/output.rs:279`) — `cmd_search()` (`src/cli_main.rs:125`) and `search_routine()` (`src/routines.rs:85`) | Same reason as above. |
| `clj .status` / `ast .journal.status` | `output::status_output()` (`src/output.rs:318`) — `cmd_status()` (`src/cli_main.rs:145`) and `status_routine()` (`src/routines.rs:102`) | Same reason as above. |
| `clj .export` / `ast .journal.export` | `output::export_output()` (`src/output.rs:461`) — `cmd_export()` (`src/cli_main.rs:151`) and `export_routine()` (`src/routines.rs:117`) | Same reason as above. |
| `clj .tail` / `ast .journal.tail` | None — `tail_routine()` (`src/routines.rs:153`) is a stub returning a fixed guidance string; it never calls `build_filter()` (`src/output.rs:82`, invoked from `cmd_tail()` at `src/cli_main.rs:101`) or `JournalReader::tail()` (`../claude_journal/src/reader.rs:116`, invoked from `cmd_tail()` at `src/cli_main.rs:108`) | Fails on two independent grounds: not a documented `command/` entry, and (even setting that aside) genuinely different behavior — `cmd_tail()` (`src/cli_main.rs:99`) performs the real tail, `tail_routine()` does not. |
| `clj .serve` / `ast .journal.serve` | None — `serve_routine()` (`src/routines.rs:171`) is a stub returning a fixed guidance string; it never binds an HTTP server | Same two-fold rejection as `.tail` above — undocumented surface plus genuinely different behavior. `cmd_serve()` (`src/cli_main.rs:161`) is the only caller of `tiny_http::Server::http()` (`src/cli_main.rs:169`) in the crate. |
| `.list` / `.stats` / `.search` / `.export` / `.tail` (intra-`clj`, no `ast` involved) | None at the dispatch-function level — only parameter *names* overlap: `type`/`since`/`command`/`format`/`until` each appear on 2–5 of these commands' own parameter tables (`docs/cli/command/01_list.md:5`, `docs/cli/command/03_stats.md:5`, `docs/cli/command/04_search.md:5`, `docs/cli/command/08_export.md:5`, `docs/cli/command/02_tail.md:5`) and are formalized as cross-command membership in `docs/cli/param_group/01_filtering.md:22-26` and `docs/cli/param_group/02_display.md` | Confirmed via the full pairwise sweep above: `cmd_list()` (`src/cli_main.rs:89`), `cmd_stats()` (`src/cli_main.rs:115`), `cmd_search()` (`src/cli_main.rs:125`), `cmd_export()` (`src/cli_main.rs:151`), and `cmd_tail()` (`src/cli_main.rs:99`) each call a distinct `output::*_output()`/inline-logic implementation with zero cross-calls between any pair (see sweep result in the Total line below). Shared parameter *names* are necessary but not sufficient — the Representation Absorption Test requires an identical **full** parameter set on top of the shared handler, and none of the five full sets match: `.list` has 12 params, `.stats` 6, `.search` 7, `.export` 6, `.tail` 5 (`docs/cli/command/readme.md:7-14`). This exact multi-command, non-identical-set relationship is what `param_group/` exists to formalize (`docs/cli/param_group/readme.md`) — it is deliberately out of command_group's stricter scope. |

No pre-existing `param_group/`-style or `parity/`-style file in this crate asserts a shared-handler claim between any two commands — unlike claude_runner's `param_group/06_running_commands.md`, there is nothing here for this entity to formalize or cross-reference; this readme is the first place any of these relationships are documented at all.

### Navigation

*(none — all 8 groups are singletons with no per-group detail file; see All Groups above)*
