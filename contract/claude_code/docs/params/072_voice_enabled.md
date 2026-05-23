# voice_enabled

Enables Claude's voice input and output capabilities.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `voiceEnabled` |

### Type

bool

### Default

`false`

### Description

When `true`, activates voice input (microphone) and audio output (text-to-speech) features. Requires platform audio support. Persists across sessions via `~/.claude/settings.json`. Has no effect in non-interactive or `--print` mode.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
