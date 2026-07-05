# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|--------|---------|-------------|----------:|
| `algorithm/` | Documented algorithms with step-by-step procedures | [algorithm/readme.md](algorithm/readme.md) | 2 |
| `cli/` | Shared CLI reference: dictionary, parameter interactions, and environment context | [cli/readme.md](cli/readme.md) | 4 |
| `cli/command/` | Per-namespace command reference for the clv CLI | [cli/command/readme.md](cli/command/readme.md) | 6 |
| `cli/format/` | Named output format catalog | [cli/format/readme.md](cli/format/readme.md) | 2 |
| `cli/param/` | Per-parameter reference for all clv parameters | [cli/param/readme.md](cli/param/readme.md) | 13 |
| `cli/param_group/` | Logical groupings of clv parameters by shared purpose | [cli/param_group/readme.md](cli/param_group/readme.md) | 4 |
| `cli/type/` | Semantic type definitions for clv parameter values | [cli/type/readme.md](cli/type/readme.md) | 8 |
| `cli/user_story/` | User story catalog with persona-goal scenarios | [cli/user_story/readme.md](cli/user_story/readme.md) | 7 |
| `feature/` | Behavioral requirements for claude_version capabilities | [feature/readme.md](feature/readme.md) | 8 |
| `pattern/` | Reusable design patterns applied in the crate | [pattern/readme.md](pattern/readme.md) | 1 |
| `pitfall/` | Confirmed design traps discovered through implementation | [pitfall/readme.md](pitfall/readme.md) | 2 |
| `runtime_file/` | On-disk files created and managed by clv at known paths | [runtime_file/readme.md](runtime_file/readme.md) | 1 |
| `tests/docs/algorithm/` | Per-algorithm test case specifications | [../tests/docs/algorithm/readme.md](../tests/docs/algorithm/readme.md) | 2 |
| `tests/docs/cli/command/` | Per-command test case specifications | [../tests/docs/cli/command/readme.md](../tests/docs/cli/command/readme.md) | 15 |
| `tests/docs/cli/format/` | Per-format output rendering test specifications | [../tests/docs/cli/format/readme.md](../tests/docs/cli/format/readme.md) | 2 |
| `tests/docs/cli/param/` | Per-parameter edge case test specifications | [../tests/docs/cli/param/readme.md](../tests/docs/cli/param/readme.md) | 13 |
| `tests/docs/cli/param_group/` | Per-group interaction test specifications | [../tests/docs/cli/param_group/readme.md](../tests/docs/cli/param_group/readme.md) | 4 |
| `tests/docs/cli/type/` | Per-type test case specifications | [../tests/docs/cli/type/readme.md](../tests/docs/cli/type/readme.md) | 8 |
| `tests/docs/cli/user_story/` | Per-user-story acceptance test specifications | [../tests/docs/cli/user_story/readme.md](../tests/docs/cli/user_story/readme.md) | 7 |
| `tests/docs/feature/` | Per-feature test case specifications | [../tests/docs/feature/readme.md](../tests/docs/feature/readme.md) | 8 |
| `tests/docs/pattern/` | Per-pattern test case specifications | [../tests/docs/pattern/readme.md](../tests/docs/pattern/readme.md) | 1 |
| `tests/docs/pitfall/` | Per-pitfall regression test case specifications | [../tests/docs/pitfall/readme.md](../tests/docs/pitfall/readme.md) | 2 |
| `tests/docs/runtime_file/` | Per-runtime-file RF- test case specifications | [../tests/docs/runtime_file/readme.md](../tests/docs/runtime_file/readme.md) | 1 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| algorithm | 001 | Settings Type Inference | [algorithm/001_settings_type_inference.md](algorithm/001_settings_type_inference.md) |
| algorithm | 002 | Config Resolution | [algorithm/002_config_resolution.md](algorithm/002_config_resolution.md) |
| cli | 002 | Dictionary | [cli/002_dictionary.md](cli/002_dictionary.md) |
| cli | 004 | Parameter Interactions | [cli/004_parameter_interactions.md](cli/004_parameter_interactions.md) |
| cli | 007 | Environment Parameters | [cli/007_env_param.md](cli/007_env_param.md) |
| cli | 008 | Config Parameters | [cli/008_config_param.md](cli/008_config_param.md) |
| cli/format | 01 | Text | [cli/format/01_text.md](cli/format/01_text.md) |
| cli/format | 02 | JSON | [cli/format/02_json.md](cli/format/02_json.md) |
| cli/user_story | 001 | Environment Check | [cli/user_story/001_environment_check.md](cli/user_story/001_environment_check.md) |
| cli/user_story | 002 | Version Upgrade | [cli/user_story/002_version_upgrade.md](cli/user_story/002_version_upgrade.md) |
| cli/user_story | 003 | Process Lifecycle | [cli/user_story/003_process_lifecycle.md](cli/user_story/003_process_lifecycle.md) |
| cli/user_story | 004 | Settings Management | [cli/user_story/004_settings_management.md](cli/user_story/004_settings_management.md) |
| cli/user_story | 005 | Version Pinning | [cli/user_story/005_version_pinning.md](cli/user_story/005_version_pinning.md) |
| cli/user_story | 006 | Config Management | [cli/user_story/006_config_management.md](cli/user_story/006_config_management.md) |
| cli/user_story | 007 | Params Inspection | [cli/user_story/007_params_inspection.md](cli/user_story/007_params_inspection.md) |
| feature | 001 | Version Management | [feature/001_version_management.md](feature/001_version_management.md) |
| feature | 002 | Process Lifecycle | [feature/002_process_lifecycle.md](feature/002_process_lifecycle.md) |
| feature | 003 | Settings Management | [feature/003_settings_management.md](feature/003_settings_management.md) |
| feature | 004 | Dry Run | [feature/004_dry_run.md](feature/004_dry_run.md) |
| feature | 005 | CLI Design | [feature/005_cli_design.md](feature/005_cli_design.md) |
| feature | 006 | Config Command | [feature/006_config_command.md](feature/006_config_command.md) |
| feature | 007 | Params Command | [feature/007_params_command.md](feature/007_params_command.md) |
| feature | 008 | Runtime File Discovery | [feature/008_runtime_file_discovery.md](feature/008_runtime_file_discovery.md) |
| pattern | 001 | Version Lock | [pattern/001_version_lock.md](pattern/001_version_lock.md) |
| pitfall | 001 | Version Lock chmod Side Effects | [pitfall/001_version_lock_chmod.md](pitfall/001_version_lock_chmod.md) |
| pitfall | 002 | Auto-Updater Symlink Retarget | [pitfall/002_symlink_retarget.md](pitfall/002_symlink_retarget.md) |
| runtime_file | 001 | Version History Cache | [runtime_file/001_version_history_cache.md](runtime_file/001_version_history_cache.md) |
| tests/docs/algorithm | 001 | Settings Type Inference | [tests/docs/algorithm/001_settings_type_inference.md](../tests/docs/algorithm/001_settings_type_inference.md) |
| tests/docs/algorithm | 002 | Config Resolution | [tests/docs/algorithm/002_config_resolution.md](../tests/docs/algorithm/002_config_resolution.md) |
| tests/docs/feature | 001 | Version Management | [tests/docs/feature/001_version_management.md](../tests/docs/feature/001_version_management.md) |
| tests/docs/feature | 002 | Process Lifecycle | [tests/docs/feature/002_process_lifecycle.md](../tests/docs/feature/002_process_lifecycle.md) |
| tests/docs/feature | 003 | Settings Management | [tests/docs/feature/003_settings_management.md](../tests/docs/feature/003_settings_management.md) |
| tests/docs/feature | 004 | Dry Run | [tests/docs/feature/004_dry_run.md](../tests/docs/feature/004_dry_run.md) |
| tests/docs/feature | 005 | CLI Design | [tests/docs/feature/005_cli_design.md](../tests/docs/feature/005_cli_design.md) |
| tests/docs/feature | 006 | Config Command | [tests/docs/feature/006_config_command.md](../tests/docs/feature/006_config_command.md) |
| tests/docs/feature | 007 | Params Command | [tests/docs/feature/007_params_command.md](../tests/docs/feature/007_params_command.md) |
| tests/docs/feature | 008 | Runtime File Discovery | [tests/docs/feature/008_runtime_file_discovery.md](../tests/docs/feature/008_runtime_file_discovery.md) |
| tests/docs/pattern | 001 | Version Lock | [tests/docs/pattern/001_version_lock.md](../tests/docs/pattern/001_version_lock.md) |
| tests/docs/pitfall | 001 | Version Lock chmod Side Effects | [tests/docs/pitfall/001_version_lock_chmod.md](../tests/docs/pitfall/001_version_lock_chmod.md) |
| tests/docs/pitfall | 002 | Auto-Updater Symlink Retarget | [tests/docs/pitfall/002_symlink_retarget.md](../tests/docs/pitfall/002_symlink_retarget.md) |
| tests/docs/runtime_file | 001 | Version History Cache | [tests/docs/runtime_file/001_version_history_cache.md](../tests/docs/runtime_file/001_version_history_cache.md) |
