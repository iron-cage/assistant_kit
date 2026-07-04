# Parameter: package_manager_auto_update

### Forms

| Form | Value |
|------|-------|
| Env Var | `CLAUDE_CODE_PACKAGE_MANAGER_AUTO_UPDATE` |

### Type

boolean (presence-activated, set to `1`)

### Default

Not set (Homebrew/WinGet installs do not auto-update)

### Description

Homebrew and WinGet installations do not auto-update by default and normally require a manual `brew upgrade` / `winget upgrade`. Setting this variable to `1` makes Claude Code run that upgrade command in the background when a new version is available and show a restart prompt on success. The upgrade targets only the Claude Code package and does not affect other installed software. On WinGet the upgrade may fail while Claude Code is running because Windows locks the executable; Claude Code falls back to showing the manual command in that case. Has no effect on native, apt, dnf, or apk installations.

### Since

v2.1.129

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [099_disable_autoupdater.md](099_disable_autoupdater.md) | Disables background auto-updates (native installs) |
| doc | [../subcommand/009_update.md](../subcommand/009_update.md) | Manual update subcommand |
