# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | Behavioral requirements for the journal viewer | [feature/readme.md](../feature/readme.md) | 3 |
| `invariant/` | Measurable constraints for the viewer | [invariant/readme.md](../invariant/readme.md) | 2 |
| `cli/command/` | CLI command specifications for the journal viewer | [cli/command/readme.md](../cli/command/readme.md) | 8 |
| `cli/param/` | CLI parameter specifications for the journal viewer | [cli/param/readme.md](../cli/param/readme.md) | 28 |
| `cli/type/` | CLI type definitions | [cli/type/readme.md](../cli/type/readme.md) | 11 |
| `cli/param_group/` | CLI parameter group definitions | [cli/param_group/readme.md](../cli/param_group/readme.md) | 5 |
| `cli/user_story/` | User story catalog for journal viewing use cases | [cli/user_story/readme.md](../cli/user_story/readme.md) | 5 |

**Total:** 7 types, 62 instances

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| feature | 001 | CLI Viewing | [feature/001_cli_viewing.md](../feature/001_cli_viewing.md) |
| feature | 002 | Web Viewing | [feature/002_web_viewing.md](../feature/002_web_viewing.md) |
| feature | 003 | Filtering | [feature/003_filtering.md](../feature/003_filtering.md) |
| invariant | 001 | Read Only | [invariant/001_read_only.md](../invariant/001_read_only.md) |
| invariant | 002 | Localhost Only | [invariant/002_localhost_only.md](../invariant/002_localhost_only.md) |
| cli/command | 01 | .list | [cli/command/01_list.md](../cli/command/01_list.md) |
| cli/command | 02 | .tail | [cli/command/02_tail.md](../cli/command/02_tail.md) |
| cli/command | 03 | .stats | [cli/command/03_stats.md](../cli/command/03_stats.md) |
| cli/command | 04 | .search | [cli/command/04_search.md](../cli/command/04_search.md) |
| cli/command | 05 | .serve | [cli/command/05_serve.md](../cli/command/05_serve.md) |
| cli/command | 06 | .prune | [cli/command/06_prune.md](../cli/command/06_prune.md) |
| cli/command | 07 | .status | [cli/command/07_status.md](../cli/command/07_status.md) |
| cli/command | 08 | .export | [cli/command/08_export.md](../cli/command/08_export.md) |
| cli/param | 01 | since | [cli/param/01_since.md](../cli/param/01_since.md) |
| cli/param | 02 | until | [cli/param/02_until.md](../cli/param/02_until.md) |
| cli/param | 03 | type | [cli/param/03_type.md](../cli/param/03_type.md) |
| cli/param | 04 | command | [cli/param/04_command.md](../cli/param/04_command.md) |
| cli/param | 05 | exit | [cli/param/05_exit.md](../cli/param/05_exit.md) |
| cli/param | 06 | model | [cli/param/06_model.md](../cli/param/06_model.md) |
| cli/param | 07 | dir | [cli/param/07_dir.md](../cli/param/07_dir.md) |
| cli/param | 08 | creds | [cli/param/08_creds.md](../cli/param/08_creds.md) |
| cli/param | 09 | limit | [cli/param/09_limit.md](../cli/param/09_limit.md) |
| cli/param | 10 | format | [cli/param/10_format.md](../cli/param/10_format.md) |
| cli/param | 11 | sort | [cli/param/11_sort.md](../cli/param/11_sort.md) |
| cli/param | 12 | reverse | [cli/param/12_reverse.md](../cli/param/12_reverse.md) |
| cli/param | 13 | by | [cli/param/13_by.md](../cli/param/13_by.md) |
| cli/param | 14 | pattern | [cli/param/14_pattern.md](../cli/param/14_pattern.md) |
| cli/param | 15 | port | [cli/param/15_port.md](../cli/param/15_port.md) |
| cli/param | 16 | bind | [cli/param/16_bind.md](../cli/param/16_bind.md) |
| cli/param | 17 | open | [cli/param/17_open.md](../cli/param/17_open.md) |
| cli/param | 18 | keep | [cli/param/18_keep.md](../cli/param/18_keep.md) |
| cli/param | 19 | dry_run | [cli/param/19_dry_run.md](../cli/param/19_dry_run.md) |
| cli/param | 20 | confirm | [cli/param/20_confirm.md](../cli/param/20_confirm.md) |
| cli/param | 21 | journal_dir | [cli/param/21_journal_dir.md](../cli/param/21_journal_dir.md) |
| cli/param | 22 | verbosity | [cli/param/22_verbosity.md](../cli/param/22_verbosity.md) |
| cli/param | 23 | output | [cli/param/23_output.md](../cli/param/23_output.md) |
| cli/param | 24 | no_color | [cli/param/24_no_color.md](../cli/param/24_no_color.md) |
| cli/param | 25 | wide | [cli/param/25_wide.md](../cli/param/25_wide.md) |
| cli/param | 26 | columns | [cli/param/26_columns.md](../cli/param/26_columns.md) |
| cli/param | 27 | refresh | [cli/param/27_refresh.md](../cli/param/27_refresh.md) |
| cli/param | 28 | include_stdout | [cli/param/28_include_stdout.md](../cli/param/28_include_stdout.md) |
| cli/type | 01 | Duration | [cli/type/01_duration.md](../cli/type/01_duration.md) |
| cli/type | 02 | EventType | [cli/type/02_event_type.md](../cli/type/02_event_type.md) |
| cli/type | 03 | String | [cli/type/03_string.md](../cli/type/03_string.md) |
| cli/type | 04 | Integer | [cli/type/04_integer.md](../cli/type/04_integer.md) |
| cli/type | 05 | Path | [cli/type/05_path.md](../cli/type/05_path.md) |
| cli/type | 06 | OutputFormat | [cli/type/06_output_format.md](../cli/type/06_output_format.md) |
| cli/type | 07 | SortField | [cli/type/07_sort_field.md](../cli/type/07_sort_field.md) |
| cli/type | 08 | Boolean | [cli/type/08_boolean.md](../cli/type/08_boolean.md) |
| cli/type | 09 | GroupBy | [cli/type/09_group_by.md](../cli/type/09_group_by.md) |
| cli/type | 10 | Port | [cli/type/10_port.md](../cli/type/10_port.md) |
| cli/type | 11 | RetentionSpec | [cli/type/11_retention_spec.md](../cli/type/11_retention_spec.md) |
| cli/param_group | 01 | Filtering | [cli/param_group/01_filtering.md](../cli/param_group/01_filtering.md) |
| cli/param_group | 02 | Display | [cli/param_group/02_display.md](../cli/param_group/02_display.md) |
| cli/param_group | 03 | Aggregation | [cli/param_group/03_aggregation.md](../cli/param_group/03_aggregation.md) |
| cli/param_group | 04 | Search | [cli/param_group/04_search.md](../cli/param_group/04_search.md) |
| cli/param_group | 05 | Global | [cli/param_group/05_global.md](../cli/param_group/05_global.md) |
| cli/user_story | 001 | Cost Tracking | [cli/user_story/001_cost_tracking.md](../cli/user_story/001_cost_tracking.md) |
| cli/user_story | 002 | Failure Diagnosis | [cli/user_story/002_failure_diagnosis.md](../cli/user_story/002_failure_diagnosis.md) |
| cli/user_story | 003 | Automation Audit | [cli/user_story/003_automation_audit.md](../cli/user_story/003_automation_audit.md) |
| cli/user_story | 004 | Capacity Planning | [cli/user_story/004_capacity_planning.md](../cli/user_story/004_capacity_planning.md) |
| cli/user_story | 005 | Team Reporting | [cli/user_story/005_team_reporting.md](../cli/user_story/005_team_reporting.md) |
