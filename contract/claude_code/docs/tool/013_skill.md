# Tool: Skill

Invoke user-defined slash command skills.

### Category

Extensibility

### Description

Executes user-defined skills (slash commands) that extend Claude Code's capabilities. Skills are defined as markdown files in the user's commands directory. Only available for skills listed in the user-invocable skills section.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [../params/021_disable_slash_commands.md](../params/021_disable_slash_commands.md) | Disable slash commands parameter |
| doc | [../params/078_disable_bundled_skills.md](../params/078_disable_bundled_skills.md) | Disable bundled skills only |
| doc | [../formats/006_command_definition.md](../formats/006_command_definition.md) | Command definition format for skill files |
| behavior | [../behavior/027_b27_agent_no_os_process.md](../behavior/027_b27_agent_no_os_process.md) | B27: contrast — Skill invocations observed as `claude --print --output-format json` OS processes; Agent subagents are not |
