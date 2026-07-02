# Testing

### Scope

- **Purpose**: Document integration and edge case test plans for all clv commands, parameters, types, and parameter groups.
- **Responsibility**: Index of per-command, per-parameter, per-type, and per-group test case planning files.
- **In Scope**: All 15 clv commands, all 13 parameters, all 8 types, all 4 parameter groups, all 7 user stories, and all 2 output formats.
- **Out of Scope**: Automated test implementations (→ `tests/` in crate), spec documentation (→ `docs/feature/`).

6-tier testing organization for `claude_version` CLI, providing distinct audience focus at each level.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `command/` | Integration test cases per command |
| `param/` | Edge case tests per parameter |
| `type/` | Type validation test cases per semantic type |
| `param_group/` | Interaction tests per parameter group |
| `format/` | Format rendering test cases per output format |
| `user_story/` | User story acceptance tests per scenario |
| `procedure.md` | Workflow for creating and updating CLI test tiers |

### Overview

| Tier | Location | Purpose | Audience | Test Types |
|------|----------|---------|----------|-----------|
| Parameter | `param/*.md` | Validate individual parameter parsing and constraints | Developers | Unit tests, edge cases |
| Type | `type/*.md` | Validate semantic type parsing, ranges, and inference | Developers | Type validation, boundary |
| Group | `param_group/*.md` | Test parameter interactions within groups | Developers | Corner cases, dependencies |
| Command | `command/*.md` | End-to-end command integration | QA / Users | Integration tests, workflows |
| User Story | `user_story/*.md` | End-to-end workflow acceptance | QA / Users | Acceptance tests, scenarios |
| Format | `format/*.md` | Output format rendering contract | Developers | Format shape, verbosity, case |

### Navigation

- [Command Tests](command/) — Integration tests per command
- [Parameter Tests](param/) — Edge case tests per parameter
- [Type Tests](type/) — Type validation tests per semantic type
- [Parameter Group Tests](param_group/) — Interaction tests per group
- [User Story Tests](user_story/) — Workflow acceptance tests per scenario
- [Format Tests](format/) — Format rendering tests per output format

### Commands
- [`.help`](command/01_help.md)
- [`.status`](command/02_status.md)
- [`.version.show`](command/03_version_show.md)
- [`.version.install`](command/04_version_install.md)
- [`.version.guard`](command/05_version_guard.md)
- [`.version.list`](command/06_version_list.md)
- [`.processes`](command/07_processes.md)
- [`.processes.kill`](command/08_processes_kill.md)
- [`.settings.show`](command/09_settings_show.md)
- [`.settings.get`](command/10_settings_get.md)
- [`.settings.set`](command/11_settings_set.md)
- [`.version.history`](command/12_version_history.md)
- [`.config`](command/13_config.md)
- [`.params`](command/14_params.md)
- [`.runtime_files`](command/15_runtime_files.md)

### Parameters
- [`version::`](param/01_version.md)
- [`dry::`](param/02_dry.md)
- [`force::`](param/03_force.md)
- [`v::` / `verbosity::`](param/04_v.md)
- [`format::`](param/05_format.md)
- [`key::`](param/06_key.md)
- [`value::`](param/07_value.md)
- [`interval::`](param/08_interval.md)
- [`count::`](param/09_count.md)
- [`.help`](param/10_help.md)
- [`scope::`](param/11_scope.md)
- [`unset::`](param/12_unset.md)
- [`kind::`](param/13_kind.md)

### Types
- [`VerbosityLevel`](type/01_verbosity_level.md)
- [`OutputFormat`](type/02_output_format.md)
- [`VersionSpec`](type/03_version_spec.md)
- [`SettingsKey`](type/04_settings_key.md)
- [`SettingsValue`](type/05_settings_value.md)
- [`ConfigScope`](type/06_config_scope.md)
- [`ConfigKey`](type/07_config_key.md)
- [`ParamKind`](type/08_param_kind.md)

### Parameter Groups
- [Output Control](param_group/01_output_control.md)
- [Execution Control](param_group/02_execution_control.md)
- [Settings Identity](param_group/03_settings_identity.md)
- [Config Identity](param_group/04_config_identity.md)

### User Stories
- [Environment Check](user_story/01_environment_check.md)
- [Version Upgrade](user_story/02_version_upgrade.md)
- [Process Lifecycle](user_story/03_process_lifecycle.md)
- [Settings Management](user_story/04_settings_management.md)
- [Version Pinning](user_story/05_version_pinning.md)
- [Config Management](user_story/06_config_management.md)
- [Params Inspection](user_story/07_params_inspection.md)

### Formats
- [Text](format/01_text.md)
- [JSON](format/02_json.md)

### Exception Records

**Exception to `cli_doc_des.rulebook.md` test ID prefix convention (`INT-` → `IT-` for command tests):**
Command integration test files use `IT-` as the test case ID prefix (e.g., `IT-1`, `IT-2`) rather than the rulebook-specified `INT-`. Rationale: `IT-` was established early and is consistent across all 14 command test files and their corresponding `tests/cli/` source functions; renaming would require synchronized changes across 14 spec files and all source function tables without any functional benefit.
