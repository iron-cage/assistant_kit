# exclude_dynamic_system_prompt_sections

Moves per-machine context out of the system prompt to improve cross-user cache reuse.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--exclude-dynamic-system-prompt-sections` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`false`

### Since

v2.1.98 (2026-04-09) — [`../version/021_v2_1_98.md`](../version/021_v2_1_98.md): *"Added `--exclude-dynamic-system-prompt-sections` flag to print mode for improved cross-user prompt caching"*

### Description

Help text:

> Move per-machine sections (cwd, env info, memory paths, git status) from the system prompt into the first user message. Improves cross-user prompt-cache reuse. Only applies with the default system prompt (ignored with `--system-prompt`). (default: false)

**Why it helps.** Prompt caching keys on a prefix. Four machine-specific sections — cwd, env info, memory paths, git status — differ per user and per checkout, so with them in the system prompt every user has a distinct prefix and nobody shares a cache entry. Relocating them into the first *user* message leaves an identical system prompt across machines, which is what makes the shared prefix cacheable.

**Silently inert with `--system-prompt`.** The help text is explicit that the flag is *ignored* when a custom system prompt is supplied — there is no default system prompt to strip sections from. Passing both is accepted at argument parsing and produces no warning, so a script that sets `--system-prompt` gains nothing from also setting this flag.

### Verification

```bash
claude --help | grep -A4 -- '--exclude-dynamic-system-prompt-sections'

for f in --exclude-dynamic-system-prompt-sections --nope-xyz; do
  claude -p "$f" </dev/null 2>&1 | grep -qi 'unknown option' \
    && echo "REJECTED $f" || echo "accepted $f"
done   # → accepted, REJECTED
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [008_append_system_prompt.md](008_append_system_prompt.md) | Appending to the default system prompt |
| doc | [051_print.md](051_print.md) | `--print` — the mode this flag targets |
| doc | [../version/021_v2_1_98.md](../version/021_v2_1_98.md) | Release introducing the flag |
