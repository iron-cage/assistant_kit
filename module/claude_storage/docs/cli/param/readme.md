# Parameters Reference

All parameters for the `claude_storage` CLI. Parameters use `param::value` syntax.

See [type/readme.md](../type/readme.md) for type definitions and [param_group/readme.md](../param_group/readme.md) for related parameter sets.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_agent.md` | agent:: — session type filter (main vs agent) |
| `02_case_sensitive.md` | case_sensitive:: — case-sensitive search toggle |
| `03_entries.md` | entries:: — show all session entries flag |
| `04_entry_type.md` | entry_type:: — filter search by entry type |
| `05_format.md` | format:: — export output format selector |
| `06_metadata.md` | metadata:: — show metadata only flag |
| `07_min_entries.md` | min_entries:: — minimum entry count threshold |
| `08_output.md` | output:: — export output file path |
| `09_path.md` | path:: — path argument (semantics vary by command) |
| `10_project.md` | project:: — project scope identifier |
| `11_query.md` | query:: — search query string (required by .search) |
| `12_scope.md` | scope:: — session/project discovery scope |
| `13_session.md` | session:: — session filter or scope pin |
| `14_session_id.md` | session_id:: — direct session identifier |
| `15_sessions.md` | sessions:: — explicit session display toggle |
| `16_target.md` | target:: — count operation target |
| `17_topic.md` | topic:: — session topic suffix |
| `18_type.md` | type:: — project naming scheme filter |
| `19_verbosity.md` | verbosity:: — output detail level |
| `20_strategy.md` | strategy:: — resume strategy override |
| `21_count.md` | count:: — output count only flag for .list |
| `22_limit.md` | limit:: — per-project session display cap |

## Parameters Table

| # | Parameter | Type | Default | Commands | Purpose |
|---|-----------|------|---------|----------|---------|
| 1 | [`agent::`](01_agent.md) | Boolean | — | 2 | Session type filter (main vs agent) |
| 2 | [`case_sensitive::`](02_case_sensitive.md) | Boolean | `0` | 1 | Case-sensitive search matching |
| 3 | [`entries::`](03_entries.md) | Boolean | `0` | 1 | Show all session entries |
| 4 | [`entry_type::`](04_entry_type.md) | [`EntryType`](../type/02_entry_type.md) | `all` | 1 | Filter search by entry type |
| 5 | [`format::`](05_format.md) | [`ExportFormat`](../type/03_export_format.md) | `markdown` | 1 | Export output format |
| 6 | [`metadata::`](06_metadata.md) | Boolean | `0` | 1 | Show metadata only mode |
| 7 | [`min_entries::`](07_min_entries.md) | [`EntryCount`](../type/01_entry_count.md) | — | 2 | Minimum entry count threshold |
| 8 | [`output::`](08_output.md) | [`StoragePath`](../type/10_storage_path.md) | — | 1 | Export output file path |
| 9 | [`path::`](09_path.md) | varies | varies | 11 | Path argument (semantics vary by command) |
| 10 | [`project::`](10_project.md) | [`ProjectId`](../type/05_project_id.md) | current dir | 5 | Project scope identifier |
| 11 | [`query::`](11_query.md) | String | — | 1 | Search query string |
| 12 | [`scope::`](12_scope.md) | [`ScopeValue`](../type/07_scope_value.md) | varies | 6 | Session/project discovery scope |
| 13 | [`session::`](13_session.md) | [`SessionFilter`](../type/08_session_filter.md) / [`SessionId`](../type/09_session_id.md) | — | 4 | Session filter or scope pin |
| 14 | [`session_id::`](14_session_id.md) | [`SessionId`](../type/09_session_id.md) | — | 2 | Direct session identifier |
| 15 | [`sessions::`](15_sessions.md) | Boolean | `0` | 1 | Explicit session display toggle |
| 16 | [`target::`](16_target.md) | [`TargetType`](../type/11_target_type.md) | `projects` | 1 | Count operation target |
| 17 | [`topic::`](17_topic.md) | [`TopicName`](../type/13_topic_name.md) | — | 4 | Session topic suffix |
| 18 | [`type::`](18_type.md) | [`ProjectType`](../type/06_project_type.md) | `all` | 1 | Project naming scheme filter |
| 19 | [`verbosity::`](19_verbosity.md) | [`VerbosityLevel`](../type/12_verbosity_level.md) | `1` | 5 | Output detail level |
| 20 | [`strategy::`](20_strategy.md) | [`StrategyType`](../type/14_strategy_type.md) | auto-detect | 1 | Resume strategy override |
| 21 | [`count::`](21_count.md) | Boolean | `0` | 1 | Output count only flag |
| 22 | [`limit::`](22_limit.md) | Integer | `0` | 1 | Per-project session display cap |

**Total:** 22 parameters
