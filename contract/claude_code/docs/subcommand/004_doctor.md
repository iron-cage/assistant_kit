# Subcommand: doctor

Check the health of your Claude Code installation.

### Usage

```
claude doctor [options]
```

### Options

| Flag | Description |
|------|-------------|
| `-h`, `--help` | Display help |

### Sub-subcommands

None.

### Description

Diagnostic command that checks the health of the Claude Code **installation** —
broader than the auto-updater alone, which is how an earlier revision of this
doc described it. The live help text in v2.1.220 reads:

> Check the health of your Claude Code installation. Reads settings files in the
> current directory without a trust prompt. For a full checkup that can also fix
> issues, run `/doctor` in a session.

Two properties worth noting, both stated in that text:

- **No trust prompt.** It reads settings files in the current directory without
  prompting for directory trust, which makes it safe to run as a first step in
  an unfamiliar checkout.
- **Read-only.** The CLI subcommand diagnoses but does not repair. The
  `/doctor` slash command inside a session is the variant that can also fix
  what it finds.

### Since

v2.0.12 (2025-10-09)

### Verification

```bash
claude doctor --help | head -3    # → the description quoted above
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [009_update.md](009_update.md) | Update/upgrade subcommand |
| doc | [../param/011_auto_updates.md](../param/011_auto_updates.md) | Auto-updates config key |
| doc | [../param/103_disable_doctor_command.md](../param/103_disable_doctor_command.md) | Env var to hide `/doctor` slash command |
