# api_key

The Anthropic API key used to authenticate all API requests.

### Forms

| | Value |
|-|-------|
| CLI Flag | — (removed from CLI; was `--api-key`) |
| Env Var | `ANTHROPIC_API_KEY` |
| Config Key | — |

### Type

string

### Default

—

### Since

pre-v1.0 (unverified)

### Description

The Anthropic API key used to authenticate requests. Must be set in the environment; the `--api-key` CLI flag was removed from the binary. Without a valid key, Claude Code falls back to browser-based OAuth authentication. For automation and CI, always set this env var explicitly rather than relying on interactive login.

**The CLI flag is gone, not merely undocumented.** v2.1.220 rejects `--api-key` outright with `unknown option`. A doc or script carrying that flag fails at argument parsing, before any request is attempted — it does not degrade to reading the env var.

**Sibling credential inputs.** Three related strings are present in the same binary, so `ANTHROPIC_API_KEY` is not the only accepted credential path:

| String | Occurrences in v2.1.220 | Role |
|--------|------------------------|------|
| `ANTHROPIC_API_KEY` | 95 | Standard API key |
| `CLAUDE_CODE_OAUTH_TOKEN` | 61 | OAuth token, for subscription-based auth |
| `ANTHROPIC_AUTH_TOKEN` | 46 | Custom `Authorization` header value |
| `apiKeyHelper` | 45 | Settings key naming a command that emits a key |

Their relative precedence is **not** established here — occurrence counts prove presence, never ordering. Nothing in this crate's tests, and no official statement cited in this collection, fixes which wins when several are set at once.

### Verification

```bash
# The flag is rejected (with a control that must also be rejected):
for f in --api-key --nonexistent-control-xyz --allowed-tools; do
  claude -p "$f" </dev/null 2>&1 | grep -qi 'unknown option' \
    && echo "REJECTED $f" || echo "accepted $f"
done   # → REJECTED, REJECTED, accepted

# The env vars are present in the binary:
V=~/.local/share/claude/versions/2.1.220
for k in ANTHROPIC_API_KEY CLAUDE_CODE_OAUTH_TOKEN ANTHROPIC_AUTH_TOKEN \
         apiKeyHelper TOTALLY_FAKE_VAR_XYZ; do
  printf '%-26s %s\n' "$k" "$(grep -ac "$k" "$V")"
done   # → 95, 61, 46, 45, 0 (last is the negative control)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [087_workspace_id.md](087_workspace_id.md) | Workspace ID for API routing |