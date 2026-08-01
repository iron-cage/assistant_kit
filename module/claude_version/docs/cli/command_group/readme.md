# Command Groups

### Scope

- **Purpose**: Formalize sets of commands that share one implementing function and one parameter set, differing only in default values.
- **Responsibility**: Define command_group membership under a strict identity test — same routine function, same parameter set — distinct from the looser cross-command relationships already noted in `command/*.md` "Related Commands" tables (e.g. `.config` as the "unified predecessor" of `.settings.*`, `.paths` as a superset of `.runtime_files`).
- **In Scope**: Group membership, the Representation Absorption Test applied per candidate pair, shared-handler citations, default divergence (when any exists), and cross-references to commands/parameters/tests/user stories.
- **Out of Scope**: Individual parameter semantics (-> `../param/`), namespace-level command reference (-> `../command/`), cross-parameter constraints (-> `../004_parameter_interactions.md`).

Every command registered in `register_commands()` (`src/lib.rs`) is evaluated against every other command using the Representation Absorption Test (see Evaluated, Not Qualifying below) before a new command name is ever added — this is a mandatory design gate, not documentation-after-the-fact. A proposed new command that would pass the test is a pre-configured alias of an existing command's routine, not a new command_group member requiring its own dispatch function.

### Overview Table

| # | Group | Members | Shared Handler | Divergence |
|---|-------|---------|-----------------|------------|
| — | *(none)* | — | — | — |

**Total:** 0 groups. All 16 documented `clv` commands were evaluated pairwise under the Representation Absorption Test. `register_commands()` (`src/lib.rs:93-107`) registers each command with its own uniquely-named routine (`status_routine`, `version_show_routine`, `version_install_routine`, `version_guard_routine`, `version_list_routine`, `version_history_routine`, `processes_routine`, `processes_kill_routine`, `settings_show_routine`, `settings_get_routine`, `settings_set_routine`, `config_routine`, `params_routine`, `runtime_files_routine`, `paths_routine`) — 15 distinct functions for 15 dispatched commands. `.help` is intercepted in `src/adapter.rs::argv_to_unilang_tokens()` before the unilang pipeline runs at all and never reaches a routine. No routine in `src/commands/*.rs` calls another routine directly, and no pair of routines shares a private implementing helper beyond the fully generic, many-caller utilities in `src/commands/mod.rs` (`is_dry`, `is_force`, `require_claude_paths`, `require_nonempty_string_arg`, `config_resolve_context`) — each of those is shared across several unrelated commands, not a single pair, so none of them constitutes a "shared handler" in the entity's sense. This is architecturally different from claude_runner's `dispatch_ask() -> dispatch_run()` delegation, which is exactly the shape this entity looks for and did not find here.

### Evaluated, Not Qualifying

| Candidate Pair | Signal That Suggested It | Why Not a Command Group |
|-----------------|---------------------------|---------------------------|
| `.settings.show` / `.settings.get` / `.settings.set` vs `.config` | `command/config.md`'s Related Commands table calls all three "Deprecated predecessor" of `.config` | `config_routine()` (`src/commands/config.rs:39`) is fully self-contained — 4-layer env/project/user/catalog resolution, its own mode dispatch (show-all/get/set/unset) and its own `render_config_*` helpers. `settings_show_routine()`/`settings_get_routine()`/`settings_set_routine()` (`src/commands/settings.rs:16,72,119`) read/write `~/.claude/settings.json` directly via `claude_core::settings_io`, with no 4-layer resolution and no reference to `config_routine` or its helpers. Parameter sets also differ: `.config` accepts `key::, value::, scope::, format::, v::, dry::, unset::` (7 params) vs `.settings.get`'s `key::, v::, format::` (3) and `.settings.set`'s `key::, value::, dry::` (3) — neither is a subset-by-defaults of the other, `.config` adds `scope::`/`unset::` with no `.settings.*` equivalent. "Predecessor" describes a product-history/successor relationship, not a shared dispatch function. |
| `.paths` vs `.runtime_files` | `command/paths.md` states `.runtime_files` reports "a subset of `.paths`'s 5" paths; `param/readme.md`/docs cross-reference each other | `paths_routine()` (`src/commands/paths.rs:97`) and `runtime_files_routine()` (`src/commands/runtime_files.rs:29`) are separate functions in separate files with no call between them. Parameter sets differ outright: `.paths` accepts `key::, format::, v::` (3 params, all optional, single-key or show-all mode) while `.runtime_files` accepts none at all and always prints one hardcoded path (`$HOME/.claude/.transient/version_history_cache.json`). A zero-parameter command can never be "the same parameter set, differing only in defaults" as a 3-parameter command — the params literally don't exist on one side to default. |
| `.status` / `.version.show` / `.version.list` / `.processes` / `.settings.show` | All five register with the identical parameter vector `[v(), fmt()]` in `register_commands()` (`src/lib.rs:93,94,97,99,101`) — the strongest doc/registration-level "same surface" signal in the crate | Registered *argument definitions* being identical is necessary but not sufficient — the Representation Absorption Test asks whether one routine's CLI-facing behavior is achievable by changing another's default parameter values, and it is not: `status_routine()` aggregates version + process count + account + lock-compliance rows from 5+ independent data sources; `version_show_routine()` reports one version string; `version_list_routine()` reports a compile-time alias table with no I/O; `processes_routine()` scans `/proc`; `settings_show_routine()` dumps `~/.claude/settings.json` key-value pairs. Each is a genuinely distinct read with a genuinely distinct output shape — no combination of `v::`/`format::` values turns one command's output into another's. Same registered args, unrelated implementations. |

### Navigation

*(none — zero groups qualify; see Evaluated, Not Qualifying above for the nearest misses)*

### See Also

- [Commands](../command/readme.md) — full per-namespace command reference (16 commands)
- [Parameter Groups](../param_group/readme.md) — parameter-level (not command-level) groupings
- [`src/lib.rs`](../../../src/lib.rs) — `register_commands()`, the ground-truth routine registration this analysis was verified against
- [`src/commands/`](../../../src/commands/) — one file per command namespace; routine implementations cited above
