# Decisions — claude_version

Design decisions and open questions for the claude_version module.

## Decision Log

| ID | Date | Status | Question | Decision |
|----|------|--------|----------|----------|
| Q-01 | 2026-06-09 | ✅ Decided | Should settings inspection use three separate commands or a unified command? | Unified `.config` command. Replaces `.settings.show/.get/.set` (deprecated). See `task/claude_version/003_config_command.md`. |
| Q-02 | 2026-06-09 | ✅ Decided | How should effective value resolution work for `.config`? | 4-layer resolution: env var → project config → user config → catalog default. See `docs/algorithm/002_config_resolution.md`. |
