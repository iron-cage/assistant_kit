# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `cli/command/` | Per-command detail pages with full parameter tables and cross-refs | [cli/command/readme.md](cli/command/readme.md) | 11 |
| `cli/format/` | Index of format doc instances covering all export rendering modes | [cli/format/readme.md](cli/format/readme.md) | 3 |
| `cli/param/` | Per-parameter detail pages with type, defaults, and command cross-refs | [cli/param/readme.md](cli/param/readme.md) | 24 |
| `cli/param_group/` | Per-group detail pages with membership, examples, and cross-refs | [cli/param_group/readme.md](cli/param_group/readme.md) | 5 |
| `cli/type/` | Per-type constraint and parsing reference | [cli/type/readme.md](cli/type/readme.md) | 13 |
| `cli/user_story/` | Index of user story instances capturing persona intent and acceptance criteria | [cli/user_story/readme.md](cli/user_story/readme.md) | 5 |
| `feature/` | Index of feature doc instances covering CLI tool scope and design decisions | [feature/readme.md](feature/readme.md) | 1 |
| `operation/` | Index of operation doc instances covering upgrade and migration procedures | [operation/readme.md](operation/readme.md) | 1 |
| `algorithm/` | Index of algorithm doc instances covering procedure design and correctness guarantees | [algorithm/readme.md](algorithm/readme.md) | 1 |
| `invariant/` | System behavioral invariants | [invariant/readme.md](invariant/readme.md) | 3 |
| `cli/pitfall/` | CLI implementation pitfall documentation | [cli/pitfall/readme.md](cli/pitfall/readme.md) | 3 |
| `tests/docs/cli/command/` | Index of per-command integration test case files covering command-level behavior | [../../tests/docs/cli/command/readme.md](../tests/docs/cli/command/readme.md) | 11 |
| `tests/docs/cli/param/` | Index of per-parameter edge case test files covering parameter-level behavior | [../../tests/docs/cli/param/readme.md](../tests/docs/cli/param/readme.md) | 24 |
| `tests/docs/cli/param_group/` | Index of per-group interaction test files covering parameter group behavior | [../../tests/docs/cli/param_group/readme.md](../tests/docs/cli/param_group/readme.md) | 5 |
| `tests/docs/cli/type/` | Index of per-type constraint test case files covering type parsing and validation | [../../tests/docs/cli/type/readme.md](../tests/docs/cli/type/readme.md) | 13 |
| `tests/docs/cli/format/` | Index of per-format output verification test case files covering export format structure | [../../tests/docs/cli/format/readme.md](../tests/docs/cli/format/readme.md) | 3 |
| `tests/docs/cli/user_story/` | Index of per-story acceptance test case files covering user story criteria | [../../tests/docs/cli/user_story/readme.md](../tests/docs/cli/user_story/readme.md) | 5 |
| `tests/docs/feature/` | FT-prefixed test spec files mirroring each `docs/feature/` instance | [../../tests/docs/feature/readme.md](../tests/docs/feature/readme.md) | 1 |
| `tests/docs/operation/` | OP-prefixed test spec files mirroring each `docs/operation/` instance | [../../tests/docs/operation/readme.md](../tests/docs/operation/readme.md) | 1 |
| `tests/docs/invariant/` | IN-prefixed contract test spec files mirroring each `docs/invariant/` instance | [../../tests/docs/invariant/readme.md](../tests/docs/invariant/readme.md) | 3 |
| `tests/docs/algorithm/` | AL-prefixed contract test spec files mirroring each `docs/algorithm/` instance | [../../tests/docs/algorithm/readme.md](../tests/docs/algorithm/readme.md) | 1 |
| `tests/docs/cli/pitfall/` | PF-prefixed contract test spec files mirroring each `docs/cli/pitfall/` instance | [../../tests/docs/cli/pitfall/readme.md](../tests/docs/cli/pitfall/readme.md) | 3 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| feature | 001 | CLI Tool | [feature/001_cli_tool.md](feature/001_cli_tool.md) |
| operation | 001 | Migration Guide | [operation/001_migration_guide.md](operation/001_migration_guide.md) |
| cli/user_story | 001 | Audit Session History | [cli/user_story/001_audit_session_history.md](cli/user_story/001_audit_session_history.md) |
| cli/user_story | 002 | Find Past Conversation | [cli/user_story/002_find_past_conversation.md](cli/user_story/002_find_past_conversation.md) |
| cli/user_story | 003 | Export Session for Review | [cli/user_story/003_export_session_for_review.md](cli/user_story/003_export_session_for_review.md) |
| cli/user_story | 004 | Query Storage Programmatically | [cli/user_story/004_query_storage_programmatically.md](cli/user_story/004_query_storage_programmatically.md) |
| cli/user_story | 005 | Resume Claude Session | [cli/user_story/005_resume_claude_session.md](cli/user_story/005_resume_claude_session.md) |
| cli/command | 01 | Status | [cli/command/01_status.md](cli/command/01_status.md) |
| cli/command | 02 | List | [cli/command/02_list.md](cli/command/02_list.md) |
| cli/command | 03 | Show | [cli/command/03_show.md](cli/command/03_show.md) |
| cli/command | 04 | Count | [cli/command/04_count.md](cli/command/04_count.md) |
| cli/command | 05 | Search | [cli/command/05_search.md](cli/command/05_search.md) |
| cli/command | 06 | Export | [cli/command/06_export.md](cli/command/06_export.md) |
| cli/command | 07 | Projects | [cli/command/07_projects.md](cli/command/07_projects.md) |
| cli/command | 08 | Project Path | [cli/command/08_project_path.md](cli/command/08_project_path.md) |
| cli/command | 09 | Project Exists | [cli/command/09_project_exists.md](cli/command/09_project_exists.md) |
| cli/command | 10 | Session Dir | [cli/command/10_session_dir.md](cli/command/10_session_dir.md) |
| cli/command | 11 | Session Ensure | [cli/command/11_session_ensure.md](cli/command/11_session_ensure.md) |
| cli/format | 01 | Markdown | [cli/format/01_markdown.md](cli/format/01_markdown.md) |
| cli/format | 02 | JSON | [cli/format/02_json.md](cli/format/02_json.md) |
| cli/format | 03 | Text | [cli/format/03_text.md](cli/format/03_text.md) |
| cli/param | 01 | Agent | [cli/param/01_agent.md](cli/param/01_agent.md) |
| cli/param | 02 | Case Sensitive | [cli/param/02_case_sensitive.md](cli/param/02_case_sensitive.md) |
| cli/param | 03 | Show Entries | [cli/param/03_entries.md](cli/param/03_entries.md) |
| cli/param | 04 | Entry Type | [cli/param/04_entry_type.md](cli/param/04_entry_type.md) |
| cli/param | 05 | Format | [cli/param/05_format.md](cli/param/05_format.md) |
| cli/param | 06 | Show Metadata | [cli/param/06_metadata.md](cli/param/06_metadata.md) |
| cli/param | 07 | Min Entries | [cli/param/07_min_entries.md](cli/param/07_min_entries.md) |
| cli/param | 08 | Output | [cli/param/08_output.md](cli/param/08_output.md) |
| cli/param | 09 | Path | [cli/param/09_path.md](cli/param/09_path.md) |
| cli/param | 10 | Project | [cli/param/10_project.md](cli/param/10_project.md) |
| cli/param | 11 | Query | [cli/param/11_query.md](cli/param/11_query.md) |
| cli/param | 12 | Scope | [cli/param/12_scope.md](cli/param/12_scope.md) |
| cli/param | 13 | Session | [cli/param/13_session.md](cli/param/13_session.md) |
| cli/param | 14 | Session ID | [cli/param/14_session_id.md](cli/param/14_session_id.md) |
| cli/param | 15 | Show Sessions | [cli/param/15_sessions.md](cli/param/15_sessions.md) |
| cli/param | 16 | Target | [cli/param/16_target.md](cli/param/16_target.md) |
| cli/param | 17 | Topic | [cli/param/17_topic.md](cli/param/17_topic.md) |
| cli/param | 18 | Type | [cli/param/18_type.md](cli/param/18_type.md) |
| cli/param | 19 | Show Stat | [cli/param/19_show_stat.md](cli/param/19_show_stat.md) |
| cli/param | 20 | Strategy | [cli/param/20_strategy.md](cli/param/20_strategy.md) |
| cli/param | 21 | Count | [cli/param/21_count.md](cli/param/21_count.md) |
| cli/param | 22 | Limit | [cli/param/22_limit.md](cli/param/22_limit.md) |
| cli/param | 23 | Show Tokens | [cli/param/23_show_tokens.md](cli/param/23_show_tokens.md) |
| cli/param | 24 | Show Tree | [cli/param/24_show_tree.md](cli/param/24_show_tree.md) |
| cli/param_group | 01 | Output Control | [cli/param_group/01_output_control.md](cli/param_group/01_output_control.md) |
| cli/param_group | 02 | Project Scope | [cli/param_group/02_project_scope.md](cli/param_group/02_project_scope.md) |
| cli/param_group | 03 | Session Identification | [cli/param_group/03_session_identification.md](cli/param_group/03_session_identification.md) |
| cli/param_group | 04 | Session Filter | [cli/param_group/04_session_filter.md](cli/param_group/04_session_filter.md) |
| cli/param_group | 05 | Scope Configuration | [cli/param_group/05_scope_configuration.md](cli/param_group/05_scope_configuration.md) |
| cli/type | 01 | Entry Count | [cli/type/01_entry_count.md](cli/type/01_entry_count.md) |
| cli/type | 02 | Entry Type | [cli/type/02_entry_type.md](cli/type/02_entry_type.md) |
| cli/type | 03 | Export Format | [cli/type/03_export_format.md](cli/type/03_export_format.md) |
| cli/type | 04 | Path Substring | [cli/type/04_path_substring.md](cli/type/04_path_substring.md) |
| cli/type | 05 | Project ID | [cli/type/05_project_id.md](cli/type/05_project_id.md) |
| cli/type | 06 | Project Type | [cli/type/06_project_type.md](cli/type/06_project_type.md) |
| cli/type | 07 | Scope Value | [cli/type/07_scope_value.md](cli/type/07_scope_value.md) |
| cli/type | 08 | Session Filter | [cli/type/08_session_filter.md](cli/type/08_session_filter.md) |
| cli/type | 09 | Session ID | [cli/type/09_session_id.md](cli/type/09_session_id.md) |
| cli/type | 10 | Storage Path | [cli/type/10_storage_path.md](cli/type/10_storage_path.md) |
| cli/type | 11 | Target Type | [cli/type/11_target_type.md](cli/type/11_target_type.md) |
| cli/type | 12 | Topic Name | [cli/type/12_topic_name.md](cli/type/12_topic_name.md) |
| cli/type | 13 | Strategy Type | [cli/type/13_strategy_type.md](cli/type/13_strategy_type.md) |
| algorithm | 001 | Agent Session Tracking | [algorithm/001_agent_session_tracking.md](algorithm/001_agent_session_tracking.md) |
| invariant | 01 | Path Encoding | [invariant/01_path_encoding.md](invariant/01_path_encoding.md) |
| invariant | 02 | Session Family | [invariant/02_session_family.md](invariant/02_session_family.md) |
| invariant | 03 | Entry Type Format | [invariant/03_entry_type_format.md](invariant/03_entry_type_format.md) |
| cli/pitfall | 01 | Parameter Validation Not Implied By Default | [cli/pitfall/01_parameter_validation.md](cli/pitfall/01_parameter_validation.md) |
| cli/pitfall | 02 | Cross-Command Bug Propagation | [cli/pitfall/02_cross_command_propagation.md](cli/pitfall/02_cross_command_propagation.md) |
| cli/pitfall | 03 | Test Data Must Match Production Format | [cli/pitfall/03_test_data_format.md](cli/pitfall/03_test_data_format.md) |
