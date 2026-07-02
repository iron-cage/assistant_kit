# output_style

Controls the visual rendering style of Claude's terminal output.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `outputStyle` |

### Type

string

### Default

`"default"`

### Since

pre-v1.0 (unverified)

### Description

Configures the visual output style used when Claude renders responses in the terminal. Distinct from `--output-format` (which controls the data serialisation format: `text`/`json`/`stream-json`); `outputStyle` is a UI-layer preference affecting visual presentation.

Known value: `"default"`. Other values are not confirmed from observed usage.

### Notes

- Config-key only; no CLI flag or env var form
- Found in project-level `.claude/settings.local.json` in practice
- Not managed by `claude_version`; written directly by Claude Code's settings UI
- Distinct from `--output-format` (`044_output_format.md`): that controls serialisation, this controls presentation

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [044_output_format.md](044_output_format.md) | Data serialisation format (text/json/stream-json) |
