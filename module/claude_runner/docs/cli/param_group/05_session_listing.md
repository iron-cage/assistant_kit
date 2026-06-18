# CLI Parameter Group: Session Listing

**Pattern:** Consumed by `dispatch_ps()` to filter rows and select columns in the `clr ps` active sessions table. Not forwarded to any subprocess.

**Purpose:** Control session listing display — filter by execution mode, select visible columns, and expand to full-width output.

### Semantic Coherence Test

"Is this flag consumed by `clr ps`, not by `run`/`ask` or the claude subprocess?" — YES for all 3.

### Why NOT Runner Control

- `--mode`, `--columns`, `--wide`: apply only to `clr ps`; never affect subprocess execution, retry behavior, or command construction — they are output display controls for the session listing command exclusively.

### Invariants

All 3 parameters are consumed by `dispatch_ps()` in `ps.rs` before table rendering. None affect subprocess execution or command construction.

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|------------|-----------------|-------|
| 6 | [`ps`](../command/06_ps.md) | Full | — | All 3 params apply; session listing command |

### Referenced Parameters

| Parameter | Type | Default | Role in Group | Description |
|-----------|------|---------|---------------|-------------|
| [`--mode`](../param/058_mode.md) | enum | `all` | Row filter | Filter sessions by execution mode (interactive/print) |
| [`--columns`](../param/059_columns.md) | string | 8 default cols | Column selector | Comma-separated list of column keys to display |
| [`--wide`](../param/060_wide.md) | bool | false | Column expander | Show all 11 columns (convenience shorthand) |

### Referenced Tests

| # | Test Spec | Scope |
|---|-----------|-------|
| 5 | [05_session_listing.md](../../../tests/docs/cli/param_group/05_session_listing.md) | Session Listing group behavior |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 26 | [026_session_listing.md](../user_story/026_session_listing.md) | Developer / CI operator |
