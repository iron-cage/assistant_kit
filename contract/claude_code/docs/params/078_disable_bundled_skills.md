# disable_bundled_skills

Disables all bundled Claude Code skills (/commands) shipped with the binary.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_DISABLE_BUNDLED_SKILLS` |
| Config Key | `disableBundledSkills` |

### Type

bool

### Default

false

### Since

v2.1.169 (2026-06-08)

### Description

When `true`, disables all slash commands and skills bundled with the Claude Code
binary. User-installed skills in `~/.claude/skills/` and project-level
`.claude/skills/` remain active. Use when you want full control over available
skills without the built-in set.

Can also be enabled via `--safe-mode` (param 77), which sets this and other
safety-related toggles.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [077_safe_mode.md](077_safe_mode.md) | Safe mode (enables this setting implicitly) |
| doc | [../tool/013_skill.md](../tool/013_skill.md) | Skill tool this disables |
