# Subcommand: auto-mode

Inspect, critique, or reset auto mode classifier configuration.

### Usage

```
claude auto-mode [command]
```

### Sub-subcommands

Two more exist beyond the two originally documented here — found via
`claude auto-mode --help` on v2.1.220, which lists 4 (plus the generic
commander.js `help [command]`, omitted as boilerplate):

| Command | Description |
|---------|-------------|
| `config` | Print the effective auto mode config as JSON: your settings where set, defaults otherwise |
| `critique [options]` | Get AI feedback on your custom auto mode rules. Options: `--model <model>` (override which model is used) |
| `defaults [options]` | Print the default auto mode environment, allow, `soft_deny`, and `hard_deny` rules as JSON |
| `reset [options]` | Reset auto mode configuration to the shipped defaults by removing the `autoMode` section from your user settings file. Options: `-y`/`--yes` (skip the confirmation prompt) |

### Description

Inspects, critiques, and can reset the configuration of the auto-mode
permission classifier. `config` shows the effective merged configuration (user
overrides + defaults); `defaults` shows the built-in default rules — note the
real rule categories are `allow`/`soft_deny`/`hard_deny`, not a generic "deny"
as an earlier revision of this doc summarized them. `critique` sends the
user's custom rules to a model for feedback rather than just inspecting
config statically. `reset` is the only sub-subcommand that mutates state —
it deletes the `autoMode` section from user settings, prompting for
confirmation unless `--yes` is passed.

Auto mode is enabled via `--permission-mode auto` or the `CLAUDE_CODE_ENABLE_AUTO_MODE`
env var. When active, a trained classifier automatically approves or denies tool
calls based on risk assessment rather than prompting the user.

### Verification

```bash
claude auto-mode --help           # → 4 real sub-subcommands + help
claude auto-mode critique --help  # → --model <model>
claude auto-mode reset --help     # → -y, --yes
```

### Since

Imprecise — corrected against the `version/` collection. No changelog entry
directly introduces the `claude auto-mode` subcommand itself. v2.1.158
(previously cited here) only records auto mode's availability expanding to
Bedrock/Vertex/Foundry (`../version/065_v2_1_158.md`) — a feature-availability
change, not the subcommand's own launch. What IS directly dated: `reset` was
added at v2.1.212, "restore the default auto-mode configuration, with a
confirmation prompt" (`../version/109_v2_1_212.md`). `config`/`defaults`/
`critique` have no dated introduction anywhere in the `version/` collection.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| doc | [../param/046_permission_mode.md](../param/046_permission_mode.md) | Permission mode including `auto` |
| doc | [../param/081_enable_auto_mode.md](../param/081_enable_auto_mode.md) | Auto mode enable env var |
| version | [../version/065_v2_1_158.md](../version/065_v2_1_158.md) | Auto mode provider-availability change, not the subcommand's own launch |
| version | [../version/109_v2_1_212.md](../version/109_v2_1_212.md) | Release that introduced `reset` |
