# User Story 007: Params Inspection

### Scope

- **Purpose**: Document the persona-goal scenario for parameter discovery and current-value inspection.
- **Responsibility**: User story for discovering what Claude Code parameters exist, how to set them, and what their current effective values are.
- **In Scope**: `.params` usage scenarios for discovery, diagnosis, and scripting.
- **Out of Scope**: Config modification (→ [006_config_management.md](006_config_management.md)), version management (→ [002_version_upgrade.md](002_version_upgrade.md)).

### Persona

**Developer (config inspector)** — a developer who wants to understand how Claude Code is currently configured, discover what parameters exist, and diagnose unexpected behavior caused by env var or config file overrides.

### Goal

Quickly discover all Claude Code parameters, see their current effective values and where those values come from, and identify which parameters can be persistently configured vs. which only apply per-invocation.

### Scenario

```
As a developer managing Claude Code across machines,
I want to inspect all configuration parameters and their current values in one place,
So that I can diagnose unexpected behavior, discover configuration options I didn't know about,
and understand the priority order (CLI > env > project config > user config > default).
```

### Primary Flow

```sh
# 1. Discover all parameters and current observable state
clv.params

# 2. Investigate a specific parameter — see all its forms and effective value
clv.params key::model

# 3. See only settings.json config params (what .config manages)
clv.params kind::config

# 4. See only env-var params — check if any unexpected overrides are active
clv.params kind::env

# 5. Get machine-readable output for scripting
clv.params format::json | jq '.[] | select(.env_value != null)'
```

### Alternate Flows

**Diagnosing an unexpected model:** User runs `claude` and it uses a different model than expected. Running `clv.params key::model` shows `CLAUDE_MODEL` is set in the env, overriding the config — the env layer wins.

**Understanding persistence:** User wants to know if `CLAUDE_CODE_BASH_TIMEOUT` can be set permanently. Running `clv.params key::bash_timeout` shows it is env-only — no config key form exists. Must be set in shell profile, not via `.config`.

### Acceptance Tests

See [tests/docs/cli/user_story/07_params_inspection.md](../../tests/docs/cli/user_story/07_params_inspection.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| command | [command/params.md](../command/params.md) | `.params` command reference |
| feature | [../../feature/007_params_command.md](../../feature/007_params_command.md) | Full behavioral spec with ACs |
| user_story | [006_config_management.md](006_config_management.md) | Config read/write user story |
