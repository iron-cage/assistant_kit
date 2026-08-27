# include_hook_events

Adds hook lifecycle events to the stream-json output.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--include-hook-events` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`false`

### Since

Unverified. No entry anywhere in the 2.1.74–2.1.220 changelog mentions `include-hook-events`, so no introduction version can be cited. What is established: the flag is accepted by v2.1.220 and listed in its `--help`. It was therefore added at or before 2.1.220 and — absent a release note — most likely before 2.1.74.

### Description

Help text:

> Include all hook lifecycle events in the output stream (only works with `--output-format=stream-json`)

**One prerequisite.** Inert unless `--output-format=stream-json`. Note this differs from [`145_forward_subagent_text.md`](145_forward_subagent_text.md), whose help text names *two* prerequisites (`--print` **and** stream-json); this flag's text names only the output format.

**Interaction with `--bare` is unverified but worth flagging.** `--bare` skips hooks entirely, so a run combining `--bare` with `--include-hook-events` has no hook lifecycle to report. Whether the binary warns, errors, or silently emits nothing is not established here.

### Verification

```bash
claude --help | grep -A3 -- '--include-hook-events'

for f in --include-hook-events --nope-xyz; do
  claude -p "$f" </dev/null 2>&1 | grep -qi 'unknown option' \
    && echo "REJECTED $f" || echo "accepted $f"
done   # → accepted, REJECTED

# The absence of a release note is itself checkable:
grep -rl 'include-hook-events' ../version/*.md   # → no output
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [031_hooks.md](031_hooks.md) | `hooks` config key — defines the lifecycle this flag reports |
| doc | [143_bare.md](143_bare.md) | `--bare` — skips hooks entirely |
| doc | [145_forward_subagent_text.md](145_forward_subagent_text.md) | Sibling stream-json content expansion |
