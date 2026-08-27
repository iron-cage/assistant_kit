# disallowed_tools

Blocks specific tools from being available for the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--disallowedTools`, `--disallowed-tools <tools...>` (both accepted; help lists camelCase first) |
| Env Var | — |
| Config Key | `disallowedTools` |

### Type

string[] (space or comma separated)

### Default

none denied

### Since

pre-v1.0 (unverified)

### Description

Blocks specific tools from being available for the session. Accepts the same tool-name format as `--allowedTools` — v2.1.220's help gives the identical example, `"Bash(git *) Edit"`. The listed tools are removed from the available set; all others remain. Useful for targeted disabling without enumerating all permitted tools.

**Two aliases, one parameter.** `--disallowedTools` and `--disallowed-tools` are the same flag, both accepted by v2.1.220, printed on one help line with camelCase first — matching the `disallowedTools` settings key.

**Deny-wins, with a caveat about scope.** Official Anthropic documentation states that `deny` rules take precedence over `allow` rules in the *permission-settings* system. It is reasonable — and this doc treats it as likely — that `--disallowedTools` subtracts from `--allowedTools` on the same principle. It is not, however, *verified*: no test in this crate exercises both flags together, and the official precedence statement is about `permissions.deny` / `permissions.allow` in settings files, not about these two CLI flags. Treat the subtraction as the expected behavior, not an established one.

**Verify:**

```bash
claude --help | grep -A2 -- '--disallowedTools'   # → both aliases, plus the example
for f in --disallowedTools --disallowed-tools --nope-xyz; do
  claude -p "$f" </dev/null 2>&1 | grep -qi 'unknown option' \
    && echo "REJECTED $f" || echo "accepted $f"
done                                              # → accepted, accepted, REJECTED
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [006_allowed_tools.md](006_allowed_tools.md) | Tool allowlist (complement) |
| doc | [068_tools.md](068_tools.md) | Full tool override (coarser control) |