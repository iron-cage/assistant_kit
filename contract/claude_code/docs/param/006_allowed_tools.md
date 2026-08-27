# allowed_tools

Restricts available tools to an explicit allowlist for the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--allowedTools`, `--allowed-tools <tools...>` (both accepted; help lists camelCase first) |
| Env Var | — |
| Config Key | `allowedTools` |

### Type

string[] (space or comma separated)

### Default

all tools enabled

### Since

pre-v1.0 (unverified)

### Description

Restricts available tools to an explicit allowlist. Tool names follow the format `ToolName` or `ToolName(pattern)` for pattern-restricted variants. The v2.1.220 help text gives the example `"Bash(git *) Edit"` — a **space** between command and glob; official Anthropic documentation writes the same construct with a colon (`Bash(git diff:*)`). Both forms appear in first-party sources; this doc does not assert which the parser canonicalizes.

Tools not listed are unavailable for the session.

**Two aliases, one parameter.** `--allowedTools` and `--allowed-tools` are the same flag, both accepted by v2.1.220. `claude --help` prints them on one line with the camelCase form first, which matches the `allowedTools` settings key.

**Unverified: interaction with `--tools`.** An earlier revision asserted "takes precedence over `--tools` when both are provided." No test or official documentation establishes that ordering, so the claim is withdrawn rather than restated. What *is* established is that the two flags are different in kind: `--tools` selects which built-in tools exist at all (help: *"Use `""` to disable all tools, `"default"` to use all tools, or specify tool names"*), while `--allowedTools` constrains what the model may invoke, including sub-command patterns. Establishing precedence requires a live session run, which no test in this crate performs.

**Verify:**

```bash
claude --help | grep -A2 -- '--allowedTools'   # → both aliases, plus the example
for f in --allowedTools --allowed-tools --nope-xyz; do
  claude -p "$f" </dev/null 2>&1 | grep -qi 'unknown option' \
    && echo "REJECTED $f" || echo "accepted $f"
done                                            # → accepted, accepted, REJECTED
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [022_disallowed_tools.md](022_disallowed_tools.md) | Tool denylist (complement) |
| doc | [068_tools.md](068_tools.md) | Full tool override (coarser control) |