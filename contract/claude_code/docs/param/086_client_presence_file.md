# client_presence_file

Path to a file whose existence signals that a client (IDE) is connected.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CLIENT_PRESENCE_FILE` |
| Config Key | — |

### Type

path

### Default

— (none)

### Since

v2.1.181

### Description

Specifies a filesystem path that Claude Code monitors for client presence
detection. When the file exists, Claude Code treats the session as
IDE-connected. Used by editor integrations to signal their connection state
without requiring a persistent socket.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [032_ide.md](032_ide.md) | IDE auto-connect |
