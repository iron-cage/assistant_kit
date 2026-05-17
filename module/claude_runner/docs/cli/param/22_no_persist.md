# Parameter :: 22. `--no-persist`

Disable session persistence. Forwards `--no-session-persistence` to the
`claude` subprocess, preventing the session from being saved to disk.

- **Type:** bool (standalone flag)
- **Default:** false (session persistence is **ON** by default; this flag turns it **OFF**)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Claude-Native Flags](../param_group.md#group--1-claude-native-flags)

```sh
clr "Fix bug"              # session saved to disk (default)
clr --no-persist "Fix bug" # session not saved; cannot be resumed
```

**Note:** Use `--no-persist` for ephemeral, stateless queries that must not
pollute session history — disposable scripted invocations, one-shot queries,
or test runs where resumability is undesired.

**Note:** Unlike `--new-session` (which starts fresh but still saves the new
session), `--no-persist` creates an entirely unsaved session.
