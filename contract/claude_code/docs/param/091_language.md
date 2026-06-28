# language

Sets the UI language for Claude Code's interface text.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `language` |

### Type

string (language code)

### Default

System locale

### Since

v2.1.176

### Description

Sets the language for Claude Code's user interface strings, status messages,
and help text. Accepts standard language codes (e.g. `"en"`, `"ja"`, `"ko"`).
Does not affect the model's response language — use `--system-prompt` for that.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [066_theme.md](066_theme.md) | UI theme configuration |
