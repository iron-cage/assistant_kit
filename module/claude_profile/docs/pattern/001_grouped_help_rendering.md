# Pattern: Grouped Column-Aligned Help Rendering

### Scope

- **Purpose**: Define the reusable rendering scheme for grouped, column-aligned `.*.help` output on CLI commands with large active parameter counts.
- **Responsibility**: Documents the presentation-group taxonomy principle, the `::`-column alignment technique, the bare-boolean-signature convention, the enum-placeholder convention, and the minimal-content policy (no version banner, no deprecated-parameter breadcrumbs) — first applied to `.accounts.help`.
- **In Scope**: Presentation-layer grouping and alignment of `.help` parameter listings; boolean/enum signature conventions in `.help` text; group header styling (color vs. bracket-free colon fallback).
- **Out of Scope**: Parameter naming/prefix conventions (→ `l0_gov.rulebook.md § Command Parameter Standards : Parameter Prefix Grouping`); cross-command semantic parameter clustering (→ `cli/param_group/`); multi-section runtime data-output separators (→ `l2_imp_universal.rulebook.md § Output Formatting : Section Headers`); runtime behavior of removed/deprecated parameters (unaffected — redirect stubs remain).

### Problem

unilang's default `.help` renderer prints one flat, alphabetically-mixed list of every parameter a command accepts. For `.accounts` this included 30 active parameters plus 19 dead REMOVED-param stubs — a 49-row wall of undifferentiated text with inconsistent signature padding and unrequested historical clutter, making the command's current surface hard to scan and hard to distinguish from its removed history.

### Solution

Render `.help` output as named, presentation-only parameter groups instead of one flat list:

1. Partition active parameters into a small number of groups scoped by *shared usage context within that command's help output* — distinct from `cli/param_group/`'s cross-command semantic clustering; a parameter's presentation group here and its `param_group/` classification may legitimately differ (see Consequences).
2. Render group headers as bold/colored text with no bracket punctuation on a TTY; fall back to a single trailing colon (e.g. `Core:`) when color is unavailable (plain-text or piped output). Reuses `cli_fmt::CliHelpStyle`'s existing `color_group` / `opt_indent` / `opt_name_width` conventions, already used unbracketed for the top-level `.` command listing.
3. Split each parameter's signature into two independently left-padded sub-columns — name, then the literal `::`, then value — so the `::` delimiter forms a straight vertical line across every row in the block.
4. Show every boolean parameter's signature bare as `name::0` (never `name::0|1`); state accepted values and default once, in a single blanket line, instead of repeating them per row.
5. Show enum-valued parameters' signature with an uppercase placeholder (e.g. `imodel::MODEL`); spell out the actual enum values in the description column, not the signature.
6. Omit entirely from `.help` output: version/build banners, and any mention of REMOVED or deprecated parameters (no footer count, no reveal flag). Removed parameters keep their existing runtime redirect-error stub behavior — a separate mechanism, unaffected by this pattern.

### Applicability

Apply to any `.*.help` output whose active parameter count is large enough that a flat list becomes hard to scan — as a rule of thumb, commands already documented with multiple `cli/param_group/` cross-cutting groups. First applied to `.accounts.help` (30 params → 6 groups); the same treatment is planned for `.usage.help`. Not applicable to commands with few enough parameters that a flat list stays scannable (e.g. `.account.save`, `.account.delete` — under 6 parameters each).

### Consequences

- Positive: help output becomes scannable at a glance; the `::` column gives a consistent visual anchor; boolean/enum signatures are shorter and non-redundant with their own description text.
- Positive: keeps `.help` output focused on the current active surface only — no historical clutter competing for attention with what a user can do today.
- Negative / cost: max line width grows (from ~105 to ~114 chars for `.accounts.help`) to accommodate independently padded `::` sub-columns — acceptable for interactive terminal use, may wrap on narrower terminals.
- Negative / cost: this command-specific *presentation* grouping is deliberately allowed to diverge from `cli/param_group/`'s cross-command *semantic* grouping (e.g. `get::` renders under "Display Rendering" in `.accounts.help`, but is classified under `param_group/001` "Output Control" for cross-command purposes). A reader consulting both must understand these are two independent, orthogonal taxonomies serving different purposes — not a single source of truth disagreeing with itself. See `cli/param_group/readme.md` for why cross-command clustering is scoped differently from this pattern.
- Relationship to `l2_imp_universal.rulebook.md § Output Formatting : Section Headers`: that rule's `titled_separator()` box-drawing format governs multi-section **runtime data** output (e.g. per-queue job listings). This pattern governs **static `.help` text** parameter grouping — an established, distinct rendering context already using `cli_fmt::CliHelpStyle`'s color-based group headers for the top-level `.` command listing. The two rules apply to different output contexts and do not conflict.
- Relationship to `l0_gov.rulebook.md § Command Parameter Standards : Parameter Prefix Grouping`: that rule governs parameter *name* prefixes (a structural/naming decision, e.g. `db_host`, `db_port`). This pattern never renames a parameter — it only changes how existing names are visually grouped in `.help` output. Pre-existing related parameters in this crate (`sort::`, `desc::`, `prefer::`, etc.) do not carry a shared literal prefix; that is a pre-existing condition of the crate's parameter-naming scheme, outside this pattern's scope and unchanged by it.

### Commands

| File | Relationship |
|------|-------------|
| [cli/command/001_account.md](../cli/command/001_account.md) | First application — `.accounts.help` (6 groups, 30 params) |

### Param Groups

| File | Relationship |
|------|-------------|
| [cli/param_group/001_output_control.md](../cli/param_group/001_output_control.md) | Cross-command semantic group; `get::` presentation group diverges (see Consequences) |
| [cli/param_group/005_display_control.md](../cli/param_group/005_display_control.md) | Cross-command semantic group; split into two presentation groups for `.accounts.help` (see Consequences) |
