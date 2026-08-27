# help

Displays help for the CLI or for a subcommand, then exits.

### Forms

| | Value |
|-|-------|
| CLI Flag | `-h`, `--help` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`false`

### Since

pre-v1.0 — no changelog entry in the 2.1.74–2.1.220 window; the flag predates it.

### Description

Help text:

> Display help for command

**Scope follows position.** `claude --help` prints the top-level help; `claude <subcommand> --help` prints that subcommand's help. In v2.1.220 the top-level output is 230 lines.

**This is the subcommand-detection primitive.** Because a real subcommand's help begins `Usage: claude <name>` while a non-subcommand falls back to the generic `Usage: claude [options] [command] [prompt]`, the first line of `claude <name> --help` discriminates the two. That is the only reliable probe: the binary emits no "unknown command" text, so grepping for one reports every string — real or not — as a valid subcommand. See [`../subcommand/readme.md`](../subcommand/readme.md) § Detecting a Subcommand.

**Short-circuits argument validation.** Like `--version`, `--help` is handled before other options are validated, which makes `claude <flag> --help` useless as a flag-acceptance probe — it prints help regardless of whether `<flag>` exists. Use `claude -p <flag> </dev/null` and discriminate on `unknown option` instead.

### Verification

```bash
claude --help | head -1              # → Usage: claude [options] [command] [prompt]
claude --help | wc -l                # → 230 on v2.1.220
claude doctor --help | head -1       # → Usage: claude doctor [options]
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [153_version.md](153_version.md) | `-v` / `--version` — the other short-circuiting flag |
| doc | [../subcommand/readme.md](../subcommand/readme.md) | Subcommand detection procedure built on this flag |
