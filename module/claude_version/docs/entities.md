# Collections

## Collection Tree

```
docs/
├── algorithm/
├── cli/
│   ├── command/          — namespace, governed by cli_doc_des.rulebook.md
│   ├── format/
│   ├── param/
│   ├── param_group/
│   ├── type/
│   └── user_story/
├── catalog/
├── feature/
├── pattern/
└── pitfall/
```

## Master Collections Table

| Type | Purpose | Master File |
|------|---------|-------------|
| `algorithm/` | Documented algorithms with step-by-step procedures | [algorithm/readme.md](algorithm/readme.md) |
| `cli/` | CLI reference: commands, parameters, types, groups | [cli/readme.md](cli/readme.md) |
| `cli/format/` | Named output format catalog | [cli/format/readme.md](cli/format/readme.md) |
| `cli/user_story/` | User story catalog with persona-goal scenarios | [cli/user_story/readme.md](cli/user_story/readme.md) |
| `catalog/` | Design decision registry | [catalog/readme.md](catalog/readme.md) |
| `feature/` | Behavioral requirements for claude_version capabilities | [feature/readme.md](feature/readme.md) |
| `pattern/` | Reusable design patterns applied in the crate | [pattern/readme.md](pattern/readme.md) |
| `pitfall/` | Confirmed design traps discovered through implementation | [pitfall/readme.md](pitfall/readme.md) |
| `tests/docs/algorithm/` | Per-algorithm test case specifications | [../../tests/docs/algorithm/readme.md](../../tests/docs/algorithm/readme.md) |
| `tests/docs/cli/command/` | Per-command test case specifications | [../../tests/docs/cli/command/readme.md](../../tests/docs/cli/command/readme.md) |
| `tests/docs/cli/format/` | Per-format output rendering test specifications | [../../tests/docs/cli/format/readme.md](../../tests/docs/cli/format/readme.md) |
| `tests/docs/cli/param/` | Per-parameter edge case test specifications | [../../tests/docs/cli/param/readme.md](../../tests/docs/cli/param/readme.md) |
| `tests/docs/cli/param_group/` | Per-group interaction test specifications | [../../tests/docs/cli/param_group/readme.md](../../tests/docs/cli/param_group/readme.md) |
| `tests/docs/cli/type/` | Per-type test case specifications | [../../tests/docs/cli/type/readme.md](../../tests/docs/cli/type/readme.md) |
| `tests/docs/cli/user_story/` | Per-user-story acceptance test specifications | [../../tests/docs/cli/user_story/readme.md](../../tests/docs/cli/user_story/readme.md) |
| `tests/docs/catalog/` | Per-decision implementation test case specifications | [../../tests/docs/catalog/readme.md](../../tests/docs/catalog/readme.md) |
| `tests/docs/feature/` | Per-feature test case specifications | [../../tests/docs/feature/readme.md](../../tests/docs/feature/readme.md) |
| `tests/docs/pattern/` | Per-pattern test case specifications | [../../tests/docs/pattern/readme.md](../../tests/docs/pattern/readme.md) |
| `tests/docs/pitfall/` | Per-pitfall regression test case specifications | [../../tests/docs/pitfall/readme.md](../../tests/docs/pitfall/readme.md) |

## Master Doc Instances Table

| Collection | ID | Name | Status | File |
|------------|-----|------|--------|------|
| algorithm | 001 | Settings Type Inference | ✅ | [algorithm/001_settings_type_inference.md](algorithm/001_settings_type_inference.md) |
| algorithm | 002 | Config Resolution | ✅ | [algorithm/002_config_resolution.md](algorithm/002_config_resolution.md) |
| cli | 001 | Commands | ✅ | [cli/command/readme.md](cli/command/readme.md) |
| cli | 002 | Dictionary | ✅ | [cli/dictionary.md](cli/dictionary.md) |
| cli | 003 | Parameter Groups | ✅ | [cli/param_group/readme.md](cli/param_group/readme.md) |
| cli | 004 | Parameter Interactions | ✅ | [cli/004_parameter_interactions.md](cli/004_parameter_interactions.md) |
| cli | 005 | Parameters | ✅ | [cli/param/readme.md](cli/param/readme.md) |
| cli | 006 | Types | ✅ | [cli/type/readme.md](cli/type/readme.md) |
| cli | 007 | Environment Parameters | ✅ | [cli/env_param.md](cli/env_param.md) |
| cli | 008 | Config Parameters | ✅ | [cli/config_param.md](cli/config_param.md) |
| catalog | 001 | Design Decisions | ✅ | [catalog/001_design_decisions.md](catalog/001_design_decisions.md) |
| feature | 001 | Version Management | ✅ | [feature/001_version_management.md](feature/001_version_management.md) |
| feature | 002 | Process Lifecycle | ✅ | [feature/002_process_lifecycle.md](feature/002_process_lifecycle.md) |
| feature | 003 | Settings Management | ✅ | [feature/003_settings_management.md](feature/003_settings_management.md) |
| feature | 004 | Dry Run | ✅ | [feature/004_dry_run.md](feature/004_dry_run.md) |
| feature | 005 | CLI Design | ✅ | [feature/005_cli_design.md](feature/005_cli_design.md) |
| feature | 006 | Config Command | ✅ | [feature/006_config_command.md](feature/006_config_command.md) |
| format | 01 | Text | ✅ | [cli/format/01_text.md](cli/format/01_text.md) |
| format | 02 | JSON | ✅ | [cli/format/02_json.md](cli/format/02_json.md) |
| pattern | 001 | Version Lock | ✅ | [pattern/001_version_lock.md](pattern/001_version_lock.md) |
| pitfall | 001 | Version Lock chmod Side Effects | ✅ | [pitfall/001_version_lock_chmod.md](pitfall/001_version_lock_chmod.md) |
| pitfall | 002 | Auto-Updater Symlink Retarget | ✅ | [pitfall/002_symlink_retarget.md](pitfall/002_symlink_retarget.md) |
| user_story | 001 | Environment Check | ✅ | [cli/user_story/001_environment_check.md](cli/user_story/001_environment_check.md) |
| user_story | 002 | Version Upgrade | ✅ | [cli/user_story/002_version_upgrade.md](cli/user_story/002_version_upgrade.md) |
| user_story | 003 | Process Lifecycle | ✅ | [cli/user_story/003_process_lifecycle.md](cli/user_story/003_process_lifecycle.md) |
| user_story | 004 | Settings Management | ✅ | [cli/user_story/004_settings_management.md](cli/user_story/004_settings_management.md) |
| user_story | 005 | Version Pinning | ✅ | [cli/user_story/005_version_pinning.md](cli/user_story/005_version_pinning.md) |
| user_story | 006 | Config Management | ✅ | [cli/user_story/006_config_management.md](cli/user_story/006_config_management.md) |
