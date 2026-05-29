# Parameters Reference

All parameters for the `claude_storage` CLI. Parameters use `param::value` syntax.

See [type/readme.md](../type/readme.md) for type definitions and [param_group/readme.md](../param_group/readme.md) for related parameter sets.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `01_agent.md` | agent:: — session type filter (main vs agent) |
| `02_case_sensitive.md` | case_sensitive:: — case-sensitive search toggle |
| `03_entries.md` | show_entries:: — show all session entries flag |
| `04_entry_type.md` | entry_type:: — filter search by entry type |
| `05_format.md` | format:: — export output format selector |
| `06_metadata.md` | show_metadata:: — show metadata only flag |
| `07_min_entries.md` | min_entries:: — minimum entry count threshold |
| `08_output.md` | output:: — export output file path |
| `09_path.md` | path:: — path argument (semantics vary by command) |
| `10_project.md` | project:: — project scope identifier |
| `11_query.md` | query:: — search query string (required by .search) |
| `12_scope.md` | scope:: — session/project discovery scope |
| `13_session.md` | session:: — session filter or scope pin |
| `14_session_id.md` | session_id:: — direct session identifier |
| `15_sessions.md` | show_sessions:: — explicit session display toggle |
| `16_target.md` | target:: — count operation target |
| `17_topic.md` | topic:: — session topic suffix |
| `18_type.md` | type:: — project naming scheme filter |
| `19_show_stat.md` | show_stat:: — session statistics footer toggle |
| `20_strategy.md` | strategy:: — resume strategy override |
| `21_count.md` | count:: — output count only flag for .list |
| `22_limit.md` | limit:: — per-project session display cap |
| `23_show_tokens.md` | show_tokens:: — token usage section toggle |
| `24_show_tree.md` | show_tree:: — agent tree-indented display toggle |

### Parameters Table

| # | Parameter | Type | Default | Valid Values | Purpose | Used In |
|---|-----------|------|---------|-------------|---------|---------|
| 1 | [`agent::`](01_agent.md) | Boolean | — | `0`, `1` | Session type filter (main vs agent) | 2 |
| 2 | [`case_sensitive::`](02_case_sensitive.md) | Boolean | `0` | `0`, `1` | Case-sensitive search matching | 1 |
| 3 | [`show_entries::`](03_entries.md) | Boolean | `0` | `0`, `1` | Show all session entries | 1 |
| 4 | [`entry_type::`](04_entry_type.md) | [`EntryType`](../type/02_entry_type.md) | `all` | `user`, `assistant`, `all` | Filter search by entry type | 1 |
| 5 | [`format::`](05_format.md) | [`ExportFormat`](../type/03_export_format.md) | `markdown` | `markdown`, `json`, `text` | Export output format | 1 |
| 6 | [`show_metadata::`](06_metadata.md) | Boolean | `0` | `0`, `1` | Show metadata only mode | 1 |
| 7 | [`min_entries::`](07_min_entries.md) | [`EntryCount`](../type/01_entry_count.md) | — | Integer ≥ 0 | Minimum entry count threshold | 2 |
| 8 | [`output::`](08_output.md) | [`StoragePath`](../type/10_storage_path.md) | — | filesystem path | Export output file path | 1 |
| 9 | [`path::`](09_path.md) | varies | varies | varies | Path argument (semantics vary by command) | 11 |
| 10 | [`project::`](10_project.md) | [`ProjectId`](../type/05_project_id.md) | current dir | path, uuid, substring | Project scope identifier | 5 |
| 11 | [`query::`](11_query.md) | String | — | any string | Search query string | 1 |
| 12 | [`scope::`](12_scope.md) | [`ScopeValue`](../type/07_scope_value.md) | varies | `local`, `relevant`, `under`, `global`, `around` | Session/project discovery scope | 6 |
| 13 | [`session::`](13_session.md) | [`SessionFilter`](../type/08_session_filter.md) / [`SessionId`](../type/09_session_id.md) | — | ID substring or exact | Session filter or scope pin | 4 |
| 14 | [`session_id::`](14_session_id.md) | [`SessionId`](../type/09_session_id.md) | — | exact session ID | Direct session identifier | 2 |
| 15 | [`show_sessions::`](15_sessions.md) | Boolean | `0` | `0`, `1` | Explicit session display toggle | 1 |
| 16 | [`target::`](16_target.md) | [`TargetType`](../type/11_target_type.md) | `projects` | `projects`, `sessions`, `entries`, `conversations` | Count operation target | 1 |
| 17 | [`topic::`](17_topic.md) | [`TopicName`](../type/13_topic_name.md) | — | identifier string | Session topic suffix | 4 |
| 18 | [`type::`](18_type.md) | [`ProjectType`](../type/06_project_type.md) | `all` | `uuid`, `path`, `all`, `conversation` | Project naming scheme filter | 1 |
| 19 | [`show_stat::`](19_show_stat.md) | Boolean | `0` | `0`, `1` | Session statistics footer in content mode | 1 |
| 20 | [`strategy::`](20_strategy.md) | [`StrategyType`](../type/14_strategy_type.md) | auto-detect | `resume`, `fresh` | Resume strategy override | 1 |
| 21 | [`count::`](21_count.md) | Boolean | `0` | `0`, `1` | Output count only flag | 1 |
| 22 | [`limit::`](22_limit.md) | Integer | `0` | Integer ≥ 0 | Per-project session display cap | 1 |
| 23 | [`show_tokens::`](23_show_tokens.md) | Boolean | `0` | `0`, `1` | Token usage section in output | 2 |
| 24 | [`show_tree::`](24_show_tree.md) | Boolean | `0` | `0`, `1` | Agent tree-indented display | 1 |

**Total:** 24 parameters
