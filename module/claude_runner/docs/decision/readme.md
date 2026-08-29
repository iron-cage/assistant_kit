# Decision Doc Entity

### Scope

- **Purpose**: Record why each design choice in the `clr` CLI was made, in enough detail that a future change can tell whether the original reason still holds.
- **Responsibility**: Index of decision doc instances covering the `--flag value` CLI redesign (task 031) and every flag-level, command-level, and pipeline-level choice made during and after it.
- **In Scope**: The decision itself, the rationale behind it, the consequences that followed, alternatives rejected, and the bug history where a decision was revised. One instance per decision, keyed by its durable `D{N}` identifier.
- **Out of Scope**: What the CLI does today — behavioral requirements (→ [`../feature/`](../feature/readme.md)), measurable constraints (→ [`../invariant/`](../invariant/readme.md)), reference surface for commands, parameters, and types (→ [`../cli/`](../cli/readme.md)). A decision explains a choice; it is not the specification of the resulting behavior.

### Overview Table

| ID | Name | Category | Purpose | Status |
|----|------|----------|---------|--------|
| D2 | [Verbose vs Quiet](002_verbose_vs_quiet.md) | Parameter Conventions | `--verbose` passes through; `--quiet` gates runner diagnostics; `--verbosity <0-5>` removed | ✅ |
| D3 | [Print Mode Requires Content](003_print_mode_requires_content.md) | Behavior | Requested print mode fails fast without a message, `--file`, or stdin content | ✅ |
| D4 | [Positional Args Joined](004_positional_args_joined.md) | Syntax | Multiple positional arguments join with spaces into one message | ✅ |
| D5 | [Unknown Flags Rejected](005_unknown_flags_rejected.md) | Parsing | Explicit whitelist; unknown flags error with a `--help` hint | ✅ |
| D6 | [Duplicate Flags Last Wins](006_duplicate_flags_last_wins.md) | Parameter Conventions | A repeated value-flag resolves to its last occurrence | ✅ |
| D7 | [Hand-Rolled Parser](007_hand_rolled_parser.md) | Parsing | No CLI framework; zero external dependencies for parsing | ✅ |
| D8 | [Three-Layer CLI Docs](008_three_layer_cli_docs.md) | Documentation | `command/` + `param/` + `type/` replaces the flat 42-file `docs/cli/` | ✅ |
| D9 | [Session Continuation By Default](009_session_continuation_default.md) | Behavior | `-c` injected automatically when a prior session exists; `--new-session` opts out | ✅ |
| D10 | [Binary Named clr](010_binary_named_clr.md) | Naming | Binary is `clr`; the crate stays `claude_runner` | ✅ |
| D11 | [Print By Default](011_print_by_default.md) | Behavior | Message, non-TTY stdin, or `--file`/stdin content routes to print mode; `--interactive` opts into TTY | ✅ |
| D12 | [Expose --system-prompt](012_expose_system_prompt.md) | Parameter Conventions | The destructive replace variant is exposed alongside `--append-system-prompt` | ✅ |
| D13 | [Commands Are Bare Words](013_commands_are_bare_words.md) | Syntax | Commands select a mode; parameters modify it — separate namespaces | ✅ |
| D14 | [Dedicated refresh Command](014_dedicated_refresh_command.md) | Behavior | `clr refresh` rather than an `isolated` invocation trick | ✅ |
| D15 | [render_summary() Gate Field](015_render_summary_gate.md) | Pipeline | Gate on the invariant `"type":"result"`, never on an optional field | ✅ |

**Status:** ✅ adopted and in effect · 🔄 adopted, implementation in progress · 🗑️ superseded.

Decisions by concern area: **Syntax**: D4, D13 | **Parsing**: D5, D7 | **Parameter Conventions**: D2, D6, D12 | **Behavior**: D3, D9, D11, D14 | **Naming**: D10 | **Pipeline**: D15 | **Documentation**: D8

**There is no D1, and the numbering starts at D2.** No decision was ever assigned D1 — the sequence began at D2 in the original notes and the gap is preserved rather than closed, because the `D{N}` identifier is cited from source comments, tests, and sibling docs. Renumbering to close the gap would silently invalidate every one of those citations. The instance filename carries the same number as the decision (`D15` → `015_render_summary_gate.md`), so an ID cited anywhere in the repo resolves to a file without a lookup table.

### Type-Specific Requirements

All `decision` doc instances must include:

1. **Title**: `# Decision: {Name}` — using `Decision` as the type prefix
2. **Identity line**: `**ID:** D{N} · **Category:** {Category} · **Status:** {Status}` — immediately under the title
3. **Scope** (H3): 4 required bullets — Purpose, Responsibility, In Scope, Out of Scope
4. **Decision** (H3): what was decided, stated as the rule now in force
5. **Rationale** (H3): why this option and not the alternatives
6. **Consequence** (H3): what changed as a result — omit only when the decision changed nothing observable
7. **Cross-References** (H3): flat table with `Type | File | Responsibility` columns

Optional sections, used where the record warrants them: **Scope Boundary** (where a decision is easily confused with an adjacent one), **History** (where a decision was revised after a bug), **Pitfall** (where a specific wrong repair is known to recur).

### Cross-Collection Dependencies

**This entity depends on**:
- `../invariant/` — several decisions delegate their behavioral specification to an invariant instance
- `../feature/` — [`../feature/006_cli_design.md`](../feature/006_cli_design.md) is the feature-level view these decisions justify

**This entity consumed by**:
- `../cli/param/` — parameter reference docs cite decisions for the reasoning behind a default
- `../../src/cli/` — source comments cite `D{N}` where a non-obvious choice needs its reason on hand
- `../../tests/` — test doc comments cite `D{N}` where the test encodes the decision
