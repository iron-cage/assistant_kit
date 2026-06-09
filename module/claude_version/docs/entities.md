# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|-----------|
| `feature/` | Behavioral requirements for claude_version capabilities | [readme.md](feature/readme.md) | 6 |
| `pattern/` | Reusable design patterns applied in the crate | [readme.md](pattern/readme.md) | 1 |
| `algorithm/` | Documented algorithms with step-by-step procedures | [readme.md](algorithm/readme.md) | 2 |
| `cli/` | CLI reference: commands, parameters, types, groups, formats, user stories | [readme.md](cli/readme.md) | 15 (8 reference + 5 user_story + 2 format) |
| standalone | Design rationale for the cm CLI redesign | — | 1 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| feature | 001 | Version Management | [feature/001_version_management.md](feature/001_version_management.md) |
| feature | 002 | Process Lifecycle | [feature/002_process_lifecycle.md](feature/002_process_lifecycle.md) |
| feature | 003 | Settings Management | [feature/003_settings_management.md](feature/003_settings_management.md) |
| feature | 004 | Dry Run | [feature/004_dry_run.md](feature/004_dry_run.md) |
| feature | 005 | CLI Design | [feature/005_cli_design.md](feature/005_cli_design.md) |
| feature | 006 | Config Command | [feature/006_config_command.md](feature/006_config_command.md) |
| pattern | 001 | Version Lock | [pattern/001_version_lock.md](pattern/001_version_lock.md) |
| algorithm | 001 | Settings Type Inference | [algorithm/001_settings_type_inference.md](algorithm/001_settings_type_inference.md) |
| algorithm | 002 | Config Resolution | [algorithm/002_config_resolution.md](algorithm/002_config_resolution.md) |
| cli | 001 | Commands | [cli/command/readme.md](cli/command/readme.md) |
| cli | 002 | Dictionary | [cli/002_dictionary.md](cli/002_dictionary.md) |
| cli | 003 | Parameter Groups | [cli/param_group/readme.md](cli/param_group/readme.md) |
| cli | 004 | Parameter Interactions | [cli/004_parameter_interactions.md](cli/004_parameter_interactions.md) |
| cli | 005 | Parameters | [cli/param/readme.md](cli/param/readme.md) |
| cli | 006 | Types | [cli/type/readme.md](cli/type/readme.md) |
| cli | 007 | Environment Parameters | [cli/env_param.md](cli/env_param.md) |
| cli | 008 | Config Parameters | [cli/config_param.md](cli/config_param.md) |
| user_story | 001 | Environment Check | [cli/user_story/001_environment_check.md](cli/user_story/001_environment_check.md) |
| user_story | 002 | Version Upgrade | [cli/user_story/002_version_upgrade.md](cli/user_story/002_version_upgrade.md) |
| user_story | 003 | Process Lifecycle | [cli/user_story/003_process_lifecycle.md](cli/user_story/003_process_lifecycle.md) |
| user_story | 004 | Settings Management | [cli/user_story/004_settings_management.md](cli/user_story/004_settings_management.md) |
| user_story | 005 | Version Pinning | [cli/user_story/005_version_pinning.md](cli/user_story/005_version_pinning.md) |
| format | 001 | text | [cli/format/01_text.md](cli/format/01_text.md) |
| format | 002 | json | [cli/format/02_json.md](cli/format/02_json.md) |
| standalone | 001 | Design Decisions | [001_design_decisions.md](001_design_decisions.md) |
