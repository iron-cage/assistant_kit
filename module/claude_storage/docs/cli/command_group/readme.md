# Command Groups

### Scope

- **Purpose**: Formalize sets of commands that share one implementing function and one parameter set, differing only in default values.
- **Responsibility**: Define command_group membership under a strict identity test — same routine function, same parameter set — distinct from the looser per-parameter cross-command groupings in `param_group/`.
- **In Scope**: Group membership, the Representation Absorption Test applied per candidate pair, shared-routine citations, default divergence (when any exists), and cross-references to commands/parameters/tests/user stories.
- **Out of Scope**: Individual parameter semantics (-> `../param/`), per-parameter cross-command groupings that don't require an identical full parameter set or a shared routine (-> `../param_group/`).

Every command in `command/` is evaluated against every other command using the Representation Absorption Test before a new command name is ever added — this is a mandatory design gate, not documentation-after-the-fact. A proposed new command that passes the test is a pre-configured alias of an existing command's routine, not a new command_group member requiring its own dispatch registration.

**Representation Absorption Test:** "Would the proposed new command be achievable by changing default values of an existing command's parameters?" A candidate pair qualifies as a `command_group` only when BOTH hold:

1. **Identical routine function** — both commands are registered in `src/cli_main.rs`'s `routines` phf map against the literal same function, or one's routine is a thin, unconditional delegate that calls the other's routine directly (not a shared lower-level helper).
2. **Identical CLI-facing parameter set** — every parameter accepted by one is accepted by the other, differing at most in default values.

A shared private helper function (e.g. `resolve_cmd_path`, `create_storage`) does NOT satisfy criterion 1 — that is ordinary code reuse, not command identity. See Evaluated, Not Qualifying below.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `readme.md` | This index — Representation Absorption Test, evaluation results, navigation |

### All Groups (0 total)

No file rows — zero command_group members were found.

**Total:** 0 groups. All 12 `claude_storage` commands are dispatched through `src/cli_main.rs`'s `routines` phf map, which maps each of the 12 command names (`.status`, `.list`, `.show`, `.tail`, `.count`, `.search`, `.export`, `.projects`, `.project.path`, `.project.exists`, `.session.dir`, `.session.ensure`) to a distinct, non-delegating routine function in `src/cli/*.rs`. No routine function calls any other routine function anywhere in the crate (verified by grepping every `*_routine(` call site against every `pub fn *_routine` definition — the only call sites are the definitions themselves and the `pub use` re-export list in `src/cli/mod.rs`). Since criterion 1 (identical routine function) fails for every one of the 66 candidate pairs, zero command_groups exist. This is a valid, complete outcome, not an incomplete audit — see `01_run_ask.md`-equivalent worked example in the sibling `claude_runner` crate for what a qualifying group looks like when the underlying dispatch mechanism does share handlers.

### Evaluated, Not Qualifying

The closest candidates — commands sharing a parameter set but NOT a routine function — are the three `session.rs`-implemented commands built on the same private path-resolution helpers:

| Candidate Pair | Shared Parameter Set | Shared Implementation | Why Not a Command Group |
|-----------------|----------------------|-------------------------|---------------------------|
| `.project.path` / `.project.exists` | `{path::, topic::}` (identical) | `resolve_cmd_path()`, `validate_topic()` (`src/cli/session.rs`) — private helpers, not the dispatch routine | Distinct routine functions (`project_path_routine` vs `project_exists_routine`, `src/cli/session.rs:69` and `:105`) implementing semantically different operations — one computes a path unconditionally (exit 0 always), the other checks existence and diverges exit code (0 vs 1) and stdout/stderr content based on filesystem state. Fails criterion 1 (identical routine) even though criterion 2 (identical parameter set) holds. |
| `.project.path` / `.session.dir` | `{path::, topic::}` (identical) | `resolve_cmd_path()` (`src/cli/session.rs`) — private helper only | Distinct routine functions (`project_path_routine` vs `session_dir_routine`, `src/cli/session.rs:69` and `:176`) computing different output paths — `.project.path` returns the Claude storage path (`~/.claude/projects/{encoded}/`) via `claude_storage_core::continuation::to_storage_path_for`; `.session.dir` returns the session working directory (`{base}/-{topic}`) via `resolve_session_dir`. Same parameter shape, unrelated output semantics. Fails criterion 1. |
| `.project.exists` / `.session.dir` | `{path::, topic::}` (identical) | `resolve_cmd_path()` (`src/cli/session.rs`) — private helper only | Distinct routine functions (`project_exists_routine` vs `session_dir_routine`, `src/cli/session.rs:105` and `:176`) with unrelated purposes (boolean history check vs. path computation). Fails criterion 1. |
| `.count` / `.search` / `.export` | Not identical — `.count` `{path, project, session, target}`, `.search` `{case_sensitive, entry_type, project, query, session}`, `.export` `{format, output, project, session_id}` | `load_project_for_param()` (`src/cli/storage.rs`) — private helper only | Fails both criteria: distinct routine functions (`count_routine`, `search_routine`, `export_routine`) AND non-identical parameter sets. Their source doc comments repeatedly note a shared historical bug pattern ("Commands .count/.search/.export shared this bug...") — this documents shared bug provenance in a common helper, not command identity; do not misread it as a shared-handler claim. |

No other pair among the remaining candidate pairs (66 total minus the 4 listed above) shares even a parameter-set match; see `command/readme.md`'s Commands Table for each command's declared parameter count.

### Navigation

*(none — zero qualifying groups; see Evaluated, Not Qualifying above for the nearest misses)*
