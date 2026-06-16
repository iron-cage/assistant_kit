# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `algorithm/` | Documented algorithms with step-by-step procedures | [algorithm/readme.md](algorithm/readme.md) | 2 |
| `cli/` | CLI reference: commands, parameters, types, groups | [cli/readme.md](cli/readme.md) | 8 |
| `collection/` | Design decision registry | [collection/readme.md](collection/readme.md) | 1 |
| `feature/` | Behavioral requirements for claude_version capabilities | [feature/readme.md](feature/readme.md) | 6 |
| `format/` | Named output format catalog | [cli/format/readme.md](cli/format/readme.md) | 2 |
| `pattern/` | Reusable design patterns applied in the crate | [pattern/readme.md](pattern/readme.md) | 1 |
| `pitfall/` | Confirmed design traps discovered through implementation | [pitfall/readme.md](pitfall/readme.md) | 2 |
| `tests/docs/algorithm/` | Per-algorithm test case specifications | [../../tests/docs/algorithm/readme.md](../../tests/docs/algorithm/readme.md) | 2 |
| `tests/docs/cli/command/` | Per-command test case specifications | [../../tests/docs/cli/command/readme.md](../../tests/docs/cli/command/readme.md) | 13 |
| `tests/docs/cli/param/` | Per-parameter edge case test specifications | [../../tests/docs/cli/param/readme.md](../../tests/docs/cli/param/readme.md) | 12 |
| `tests/docs/cli/param_group/` | Per-group interaction test specifications | [../../tests/docs/cli/param_group/readme.md](../../tests/docs/cli/param_group/readme.md) | 4 |
| `tests/docs/cli/type/` | Per-type test case specifications | [../../tests/docs/cli/type/readme.md](../../tests/docs/cli/type/readme.md) | 7 |
| `tests/docs/feature/` | Per-feature test case specifications | [../../tests/docs/feature/readme.md](../../tests/docs/feature/readme.md) | 6 |
| `tests/docs/pattern/` | Per-pattern test case specifications | [../../tests/docs/pattern/readme.md](../../tests/docs/pattern/readme.md) | 1 |
| `user_story/` | User story catalog with persona-goal scenarios | [cli/user_story/readme.md](cli/user_story/readme.md) | 5 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| algorithm | 001 | Settings Type Inference | [algorithm/001_settings_type_inference.md](algorithm/001_settings_type_inference.md) |
| algorithm | 002 | Config Resolution | [algorithm/002_config_resolution.md](algorithm/002_config_resolution.md) |
| cli | 001 | Commands | [cli/command/readme.md](cli/command/readme.md) |
| cli | 002 | Dictionary | [cli/dictionary.md](cli/dictionary.md) |
| cli | 003 | Parameter Groups | [cli/param_group/readme.md](cli/param_group/readme.md) |
| cli | 004 | Parameter Interactions | [cli/004_parameter_interactions.md](cli/004_parameter_interactions.md) |
| cli | 005 | Parameters | [cli/param/readme.md](cli/param/readme.md) |
| cli | 006 | Types | [cli/type/readme.md](cli/type/readme.md) |
| cli | 007 | Environment Parameters | [cli/env_param.md](cli/env_param.md) |
| cli | 008 | Config Parameters | [cli/config_param.md](cli/config_param.md) |
| collection | 001 | Design Decisions | [collection/001_design_decisions.md](collection/001_design_decisions.md) |
| feature | 001 | Version Management | [feature/001_version_management.md](feature/001_version_management.md) |
| feature | 002 | Process Lifecycle | [feature/002_process_lifecycle.md](feature/002_process_lifecycle.md) |
| feature | 003 | Settings Management | [feature/003_settings_management.md](feature/003_settings_management.md) |
| feature | 004 | Dry Run | [feature/004_dry_run.md](feature/004_dry_run.md) |
| feature | 005 | CLI Design | [feature/005_cli_design.md](feature/005_cli_design.md) |
| feature | 006 | Config Command | [feature/006_config_command.md](feature/006_config_command.md) |
| pattern | 001 | Version Lock | [pattern/001_version_lock.md](pattern/001_version_lock.md) |
| pitfall | 001 | Version Lock chmod Side Effects | [pitfall/001_version_lock_chmod.md](pitfall/001_version_lock_chmod.md) |
| pitfall | 002 | Auto-Updater Symlink Retarget | [pitfall/002_symlink_retarget.md](pitfall/002_symlink_retarget.md) |
| user_story | 001 | Environment Check | [cli/user_story/001_environment_check.md](cli/user_story/001_environment_check.md) |
| user_story | 002 | Version Upgrade | [cli/user_story/002_version_upgrade.md](cli/user_story/002_version_upgrade.md) |
| user_story | 003 | Process Lifecycle | [cli/user_story/003_process_lifecycle.md](cli/user_story/003_process_lifecycle.md) |
| user_story | 004 | Settings Management | [cli/user_story/004_settings_management.md](cli/user_story/004_settings_management.md) |
| user_story | 005 | Version Pinning | [cli/user_story/005_version_pinning.md](cli/user_story/005_version_pinning.md) |
