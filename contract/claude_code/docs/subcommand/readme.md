# Subcommand Doc Entity

All subcommands exposed by the `claude` binary beyond the default interactive/print session mode.

### Scope

- **Purpose**: Authoritative reference for every subcommand the `claude` binary provides.
- **Responsibility**: Master table and per-subcommand detail files.
- **In Scope**: All 19 subcommands present in v2.1.220 — the 12 listed in `claude --help`, plus 7 that are functional but hidden from it.
- **Out of Scope**: The default session mode (→ `../param/`); builder-API (→ `module/claude_runner_core/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| readme.md | Master subcommand table (this file) |
| 001_agents.md | `claude agents` — manage background agents |
| 002_auth.md | `claude auth` — manage authentication |
| 003_auto_mode.md | `claude auto-mode` — inspect auto mode classifier |
| 004_doctor.md | `claude doctor` — check installation health |
| 005_install.md | `claude install` — install native build |
| 006_mcp.md | `claude mcp` — configure MCP servers |
| 007_plugin.md | `claude plugin` — manage plugins |
| 008_setup_token.md | `claude setup-token` — set up auth token |
| 009_update.md | `claude update` — check for and install updates |
| 010_gateway.md | `claude gateway` — run the enterprise auth/telemetry gateway |
| 011_project.md | `claude project` — manage and purge per-project state |
| 012_ultrareview.md | `claude ultrareview` — cloud multi-agent code review |
| 013_attach.md | `claude attach` — open a background session in this terminal |
| 014_daemon.md | `claude daemon` — manage the background-session supervisor |
| 015_import.md | `claude import` — import config from another coding agent |
| 016_logs.md | `claude logs` — print a background session's recent output |
| 017_respawn.md | `claude respawn` — restart sessions onto the current binary |
| 018_rm.md | `claude rm` — delete a background session and its worktree |
| 019_stop.md | `claude stop` — stop a background session, keeping its conversation |

### Subcommand Table

Listed in `claude --help` (12):

| # | Subcommand | Sub-subcommands | Since | Description |
|---|------------|-----------------|-------|-------------|
| 1 | [agents](001_agents.md) | — | v1.0.60 | Manage background agents — the "agent view" (17 options as of v2.1.220) |
| 2 | [auth](002_auth.md) | `login`, `logout`, `status` | pre-v1.0 | Manage authentication — sign in, sign out, show status |
| 3 | [auto-mode](003_auto_mode.md) | `config`, `critique`, `defaults`, `reset` | unverified | Inspect, critique, or reset auto mode classifier configuration |
| 4 | [doctor](004_doctor.md) | — | v2.0.12 | Check the health of your Claude Code installation |
| 5 | [gateway](010_gateway.md) | — | unverified | Run the enterprise auth/telemetry gateway |
| 6 | [install](005_install.md) | — | pre-v1.0 | Install Claude Code native build (`stable`, `latest`, or specific version) |
| 7 | [mcp](006_mcp.md) | `add`, `add-from-claude-desktop`, `add-json`, `get`, `list`, `login`, `logout`, `remove`, `reset-project-choices`, `serve` | pre-v1.0 | Configure and manage MCP servers |
| 8 | [plugin](007_plugin.md) | `details`, `disable`, `enable`, `eval`, `init`, `install`, `list`, `marketplace`, `prune`, `tag`, `uninstall`, `update`, `validate` | v2.0.12 | Manage Claude Code plugins — alias `plugins` |
| 9 | [project](011_project.md) | `purge`, `help` | v2.1.126 | Manage Claude Code project state |
| 10 | [setup-token](008_setup_token.md) | — | pre-v1.0 | Set up a long-lived authentication token (requires Claude subscription) |
| 11 | [ultrareview](012_ultrareview.md) | — | v2.1.120 | Cloud-hosted multi-agent code review of the current branch or a PR |
| 12 | [update](009_update.md) | — | pre-v1.0 | Check for updates and install if available — alias `upgrade` |

### Hidden Subcommands

Seven further subcommands are fully functional in v2.1.220 but do not appear in
`claude --help`. Six of them form the background-session lifecycle family;
`import` is unrelated.

| # | Subcommand | Sub-subcommands | Since | Description |
|---|------------|-----------------|-------|-------------|
| 13 | [attach](013_attach.md) | — | unverified | Open a background session in this terminal |
| 14 | [daemon](014_daemon.md) | `run`, `status`, `logs`, `uninstall`, `stop` | unverified | Manage the background-session supervisor |
| 15 | [import](015_import.md) | — | unverified | Import config from another coding agent (`codex`, `gemini`) |
| 16 | [logs](016_logs.md) | — | unverified | Print a background session's recent terminal output |
| 17 | [respawn](017_respawn.md) | — | unverified | Restart a session (or `--all`) onto the current binary |
| 18 | [rm](018_rm.md) | — | unverified | Delete a background session and its worktree |
| 19 | [stop](019_stop.md) | — | unverified | Stop a background session, keeping its conversation |

`Since: unverified` means no changelog entry in [`../version/`](../version/readme.md)
records the subcommand's introduction. Several appear in later bug-fix entries
(`claude daemon status` at v2.1.141, `claude respawn` at v2.1.144,
`claude attach` at v2.1.198), which bound their existence from above but do not
date it.

`stop` additionally has an undocumented alias, `kill` — not counted as a 20th
subcommand since it is not a distinct capability, same as `plugins` and
`upgrade` are not counted as separate entries from `plugin` and `update`. See
[019_stop.md](019_stop.md) for why it resolves differently from those two.

### Detecting a Subcommand

`claude <name> --help` does **not** error on an unknown name — it silently falls
back to printing the top-level help, so an unknown name looks identical to a
valid one at the exit-code level. The discriminator is the first line of output:

```bash
claude agents --help | head -1             # → Usage: claude agents [options]
claude totallyfakecmdxyz --help | head -1  # → Usage: claude [options] [command] [prompt]
```

A real subcommand prints `Usage: claude <name>`; anything else prints the
generic top-level usage line. Always include a fabricated name as a negative
control — without one, a probe that reports every name as valid is
indistinguishable from a working probe.

Aliases resolve to the same usage line as their canonical name — but that line
lists both names joined by `|`, canonical first, not the canonical name alone:
`claude plugins --help` prints `Usage: claude plugin|plugins [options] [command]`,
and `claude upgrade --help` prints `Usage: claude update|upgrade [options]`
(live-verified against v2.1.220; contrast a non-aliased subcommand like `auth`,
whose usage line is the single name `Usage: claude auth [options] [command]`).
A detector matching only the exact string `Usage: claude plugin` would miss
this — match the prefix, or split on `|`, instead. Under the "first line"
detection rule above, aliases still read as "absent from `--help`" while being
perfectly valid to type; they are aliases, not separate subcommands.

That `|`-splitting heuristic is not exhaustive, though: `claude kill --help`
resolves to `Usage: claude stop <id>` — byte-identical to `stop`'s own output,
with no `|kill` joined in (see [019_stop.md](019_stop.md)). A detector relying
only on the presence of `|` would misclassify `kill` as a non-alias, unrelated
command rather than recognizing it as `stop` under another name. The fully
general test is comparing the *entire* returned help text against every known
canonical output, not just checking for a pipe character.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [`../param/readme.md`](../param/readme.md) | CLI parameter specifications |
| doc | [`../tool/readme.md`](../tool/readme.md) | Built-in tools available in sessions |
| doc | [`../version/readme.md`](../version/readme.md) | Release changelog used to date subcommand introductions |
