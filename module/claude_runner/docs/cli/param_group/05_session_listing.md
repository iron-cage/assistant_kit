# CLI Parameter Group: Session Listing

**Pattern:** Consumed by `dispatch_ps()` to filter rows, select columns, and control output format in the `clr ps` active sessions table. Not forwarded to any subprocess.

**Purpose:** Control session listing display — filter by execution mode, filter by PID, select visible columns, expand to full-width output, or switch to key:value inspect format.
**Order:** 5

### Semantic Coherence Test

"Is this flag consumed by `clr ps`, not by `run`/`ask` or the claude subprocess?" — YES for all 5.

### Why NOT Runner Control

- `--mode`, `--columns`, `--wide`, `--pid`, `--inspect`: apply only to `clr ps`; never affect subprocess execution, retry behavior, or command construction — they are output display controls for the session listing command exclusively.

### Invariants

All 5 parameters are consumed by `dispatch_ps()` in `ps.rs` before table rendering. None affect subprocess execution or command construction.

### Notes

—

### Typical Patterns

```sh
clr ps
clr ps --mode print
clr ps --columns pid,path,task
clr ps --wide --mode interactive
clr ps --pid 1234567 --inspect
```

### Referenced Commands

| # | Command | Membership | Excluded Params | Notes |
|---|---------|------------|-----------------|-------|
| 6 | [`ps`](../command/06_ps.md) | Full | — | All 5 params apply; session listing command |

### Referenced Parameters

| Parameter | Type | Default | Role in Group | Description |
|-----------|------|---------|---------------|-------------|
| [`--mode`](../param/058_mode.md) | enum | `all` | Row filter | Filter sessions by execution mode (interactive/print) |
| [`--columns`](../param/059_columns.md) | string | 9 default cols | Column selector | Comma-separated list of column keys to display |
| [`--wide`](../param/060_wide.md) | bool | false | Column expander | Show all 11 columns (convenience shorthand) |
| [`--pid`](../param/068_pid.md) | string | — | PID filter | Restrict output to comma-separated list of process IDs |
| [`--inspect`](../param/069_inspect.md) | bool | false | Output mode | Switch to key:value record format showing all 12 attributes |

### Referenced Tests

| # | Test Spec | Scope |
|---|-----------|-------|
| 5 | [05_session_listing.md](../../../tests/docs/cli/param_group/05_session_listing.md) | Session Listing group behavior |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 26 | [026_session_listing.md](../user_story/026_session_listing.md) | Developer / CI operator |
