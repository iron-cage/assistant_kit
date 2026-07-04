# Algorithm Doc Entity

### Scope

- **Purpose**: Test case specifications for claude_version algorithm doc instances.
- **Responsibility**: Per-algorithm test specs verifying algorithm steps and edge cases.
- **In Scope**: Algorithm test planning (AC- prefix, min 4 cases per spec).
- **Out of Scope**: Feature tests (-> `feature/`), CLI command tests (-> `cli/command/`).

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| 001_settings_type_inference.md | AC- test cases for 4-step type inference cascade | ✅ |
| 002_config_resolution.md | AC- test cases for 4-layer resolution algorithm (env/project/user/default) | ✅ |
| procedure.md | Workflow for creating and updating algorithm test specs | ✅ |
