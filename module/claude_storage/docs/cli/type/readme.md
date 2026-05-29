# Type System

Semantic newtypes for `claude_storage` CLI parameters. Every parameter uses a named type with validation constraints — never bare primitives.

See [param/readme.md](../param/readme.md) for which parameters use each type.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_entry_count.md` | EntryCount — non-negative integer for entry thresholds |
| `02_entry_type.md` | EntryType — author filter enum (user/assistant/all) |
| `03_export_format.md` | ExportFormat — serialization format enum (markdown/json/text) |
| `04_path_substring.md` | PathSubstring — case-insensitive path filter string |
| `05_project_id.md` | ProjectId — multi-format project identifier |
| `06_project_type.md` | ProjectType — project naming scheme enum (path/uuid/all) |
| `07_scope_value.md` | ScopeValue — discovery boundary enum (local/relevant/under/global/around) |
| `08_session_filter.md` | SessionFilter — case-insensitive session ID substring |
| `09_session_id.md` | SessionId — exact session identifier |
| `10_storage_path.md` | StoragePath — filesystem path (absolute or ~-prefixed) |
| `11_target_type.md` | TargetType — count target enum (projects/sessions/entries) |
| `13_topic_name.md` | TopicName — session topic identifier string |
| `14_strategy_type.md` | StrategyType — resume strategy enum (resume/fresh) |

### Type Index

| # | Type | Fundamental | Used By |
|---|------|-------------|---------|
| 1 | [`EntryCount`](01_entry_count.md) | Integer (≥0) | `min_entries::` |
| 2 | [`EntryType`](02_entry_type.md) | String enum | `entry_type::` |
| 3 | [`ExportFormat`](03_export_format.md) | String enum | `format::` |
| 4 | [`PathSubstring`](04_path_substring.md) | String | `path::` in `.list` |
| 5 | [`ProjectId`](05_project_id.md) | String (multi-format) | `project::` |
| 6 | [`ProjectType`](06_project_type.md) | String enum | `type::` |
| 7 | [`ScopeValue`](07_scope_value.md) | String enum | `scope::` |
| 8 | [`SessionFilter`](08_session_filter.md) | String | `session::` |
| 9 | [`SessionId`](09_session_id.md) | String | `session_id::`, `session::` |
| 10 | [`StoragePath`](10_storage_path.md) | String (filesystem) | `path::` (most), `output::` |
| 11 | [`TargetType`](11_target_type.md) | String enum | `target::` |
| 12 | [`TopicName`](13_topic_name.md) | String (identifier) | `topic::` |
| 13 | [`StrategyType`](14_strategy_type.md) | String enum | `strategy::` |
| 14 | [`StrategyType`](14_strategy_type.md) | String enum | `strategy::` |

### Navigation

- [EntryCount](01_entry_count.md)
- [EntryType](02_entry_type.md)
- [ExportFormat](03_export_format.md)
- [PathSubstring](04_path_substring.md)
- [ProjectId](05_project_id.md)
- [ProjectType](06_project_type.md)
- [ScopeValue](07_scope_value.md)
- [SessionFilter](08_session_filter.md)
- [SessionId](09_session_id.md)
- [StoragePath](10_storage_path.md)
- [TargetType](11_target_type.md)
- [TopicName](13_topic_name.md)
- [StrategyType](14_strategy_type.md)
