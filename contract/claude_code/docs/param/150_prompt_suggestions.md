# prompt_suggestions

Emits a predicted next user prompt after each turn.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--prompt-suggestions [value]` |
| Env Var | — |
| Config Key | — |

### Type

bool-like enum — accepts `"true"`, `"false"`, `"1"`, `"0"`, `"yes"`, `"no"`, `"on"`, `"off"`

### Default

off when the flag is absent; **preset `"true"`** when the flag is given with no value

### Since

Unverified. No entry in the 2.1.74–2.1.220 changelog mentions `prompt-suggestions`. Accepted by v2.1.220 and listed in its `--help`.

### Description

Help text:

> Enable prompt suggestions. In print/SDK mode, emits a `prompt_suggestion` message after each turn with a predicted next user prompt (choices: "true", "false", "1", "0", "yes", "no", "on", "off", preset: "true")

**The optional value is the notable part.** The bracketed `[value]` means the flag may be passed bare — in which case the *preset* `"true"` applies — or with any of the eight listed spellings. `--prompt-suggestions false` is therefore a meaningful, distinct invocation from omitting the flag only if some other source (a setting, a default) would otherwise have enabled it. This is the only parameter in this collection documented with an explicit multi-spelling boolean vocabulary.

**Output shape in print/SDK mode.** A `prompt_suggestion` message is appended after each turn. Consumers parsing the message stream must tolerate this additional message type, or they will treat it as unrecognized.

### Verification

```bash
claude --help | grep -A6 -- '--prompt-suggestions'   # → the choices and preset

for f in --prompt-suggestions --nope-xyz; do
  claude -p "$f" </dev/null 2>&1 | grep -qi 'unknown option' \
    && echo "REJECTED $f" || echo "accepted $f"
done   # → accepted, REJECTED
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [051_print.md](051_print.md) | `--print` — one of the two modes that emit the message |
| doc | [../jsonl/readme.md](../jsonl/readme.md) | Message/entry types in the output stream |
