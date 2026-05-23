# Types

Domain types used by `clp` CLI parameters and commands.

| File | Responsibility |
|------|----------------|
| [001_account_name.md](001_account_name.md) | `AccountName` newtype: email-keyed credential identifier |
| [002_output_format.md](002_output_format.md) | `OutputFormat` enum: text/json/table serialization selector |
| [003_warning_threshold.md](003_warning_threshold.md) | `WarningThreshold` newtype: token expiry classification boundary |
| [004_account_selector.md](004_account_selector.md) | `AccountSelector`: pre-resolution account identification forms |

**Total:** 4 types

### Overview Table

| # | Type | Fundamental | Parameters | Commands |
|---|------|-------------|------------|----------|
| 1 | `AccountName` | `String` (newtype) | [`name::`](../param/001_name.md) | 5 cmds |
| 2 | `OutputFormat` | `enum` | [`format::`](../param/002_format.md) | 6 cmds |
| 3 | `WarningThreshold` | `u64` (newtype) | [`threshold::`](../param/003_threshold.md) | 1 cmd |
| 4 | `AccountSelector` | logical (adapter-layer) | [`name::`](../param/001_name.md) | 4 cmds |

### See Also

- [../param/](../param/readme.md) — parameters that use these types
- [../command/](../command/readme.md) — commands that accept these types
