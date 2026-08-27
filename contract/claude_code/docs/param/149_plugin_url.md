# plugin_url

Fetches a plugin `.zip` from a URL for the current session only.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--plugin-url <url>` (repeatable) |
| Env Var | — |
| Config Key | — |

### Type

string[] (URL, repeatable)

### Default

`[]`

### Since

v2.1.129 (2026-05-06) — [`../version/042_v2_1_129.md`](../version/042_v2_1_129.md): *"Added `--plugin-url` flag to fetch a plugin `.zip` archive from a URL for the current session"*

### Description

Help text:

> Fetch a plugin .zip from a URL for this session only (repeatable: `--plugin-url A --plugin-url B`) (default: [])

**Session-scoped, not installed.** The plugin applies to the invoking session and is not registered in `enabledPlugins`. A subsequent session without the flag does not have it.

**Repeatable, with an array default.** The `(default: [])` in the help text confirms accumulation rather than last-wins: passing the flag twice loads two plugins.

**Security surface worth stating plainly.** This fetches and loads executable extension code from an arbitrary URL. The flag makes no claim about signature verification or origin pinning, and none is documented. Treat every `--plugin-url` argument with the same trust you would give to running the archive's contents directly.

### Verification

```bash
claude --help | grep -A3 -- '--plugin-url'

for f in --plugin-url --nope-xyz; do
  claude -p "$f" </dev/null 2>&1 | grep -qi 'unknown option' \
    && echo "REJECTED $f" || echo "accepted $f"
done   # → accepted, REJECTED
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [024_enabled_plugins.md](024_enabled_plugins.md) | `enabledPlugins` config key — the persistent registry this bypasses |
| doc | [143_bare.md](143_bare.md) | `--bare` — skips plugin sync; `--plugin-dir` is its explicit alternative |
| doc | [../version/042_v2_1_129.md](../version/042_v2_1_129.md) | Release introducing the flag |
