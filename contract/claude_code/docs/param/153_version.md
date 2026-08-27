# version

Prints the version number and exits.

### Forms

| | Value |
|-|-------|
| CLI Flag | `-v`, `--version` |
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

> Output the version number

**Short-circuits option parsing — do not build probes on it.** `--version` is handled before other arguments are validated, so `claude <flag> --version` prints the version whether or not `<flag>` exists. An earlier revision of this collection used exactly that construct to test flag acceptance and drew wrong conclusions from it. The valid probe is:

```bash
claude -p <flag> </dev/null 2>&1 | grep -qi 'unknown option'
```

discriminating on the string `unknown option`, with a known-bad control in the same loop to prove the probe can fail.

**Reports the running binary, not the installed set.** Claude Code keeps multiple versions under `~/.local/share/claude/versions/`; `--version` reports whichever one is on `PATH`. When binary string-scans are cited as evidence in this collection, they name the scanned version explicitly for that reason.

### Verification

```bash
claude --version                                  # → e.g. 2.1.220 (Claude Code)
ls ~/.local/share/claude/versions/                # → the installed set

# Demonstrate the short-circuit (this is the anti-pattern, shown so it stays known):
claude --nonexistent-control-xyz --version        # → prints a version, no error
claude -p --nonexistent-control-xyz </dev/null    # → error: unknown option
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [146_help.md](146_help.md) | `-h` / `--help` — the other short-circuiting flag |
| doc | [../version/readme.md](../version/readme.md) | Release changelog collection |
| doc | [../pattern/readme.md](../pattern/readme.md) | Version-pinning design pattern |
