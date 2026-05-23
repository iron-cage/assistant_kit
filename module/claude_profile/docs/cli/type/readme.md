# Types

Domain types used by `clp` CLI parameters and commands.

| File | Responsibility |
|------|----------------|
| [01_account_name.md](01_account_name.md) | `AccountName` newtype: email-keyed credential identifier |
| [02_output_format.md](02_output_format.md) | `OutputFormat` enum: text/json/table serialization selector |
| [03_warning_threshold.md](03_warning_threshold.md) | `WarningThreshold` newtype: token expiry classification boundary |
| [04_account_selector.md](04_account_selector.md) | `AccountSelector`: pre-resolution account identification forms |

**Total:** 4 types

### Overview Table

| # | Type | Fundamental | Parameters | Commands |
|---|------|-------------|------------|----------|
| 1 | `AccountName` | `String` (newtype) | [`name::`](../param/01_name.md) | 5 cmds |
| 2 | `OutputFormat` | `enum` | [`format::`](../param/02_format.md) | 6 cmds |
| 3 | `WarningThreshold` | `u64` (newtype) | [`threshold::`](../param/03_threshold.md) | 1 cmd |
| 4 | `AccountSelector` | logical (adapter-layer) | [`name::`](../param/01_name.md) | 4 cmds |

### See Also

- [../param/](../param/readme.md) — parameters that use these types
- [../command/](../command/readme.md) — commands that accept these types
