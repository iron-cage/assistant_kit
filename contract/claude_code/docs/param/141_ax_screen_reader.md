# ax_screen_reader

Renders flat, screen-reader-friendly output instead of the decorated TUI.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--ax-screen-reader` |
| Env Var | `CLAUDE_AX_SCREEN_READER` (set to `1`) |
| Config Key | `axScreenReader` |

### Type

bool

### Default

`false`

### Since

v2.1.208 (2026-07-14) — [`../version/105_v2_1_208.md`](../version/105_v2_1_208.md)

### Description

Opt-in accessibility mode. The v2.1.220 help text describes it as:

> Render screen-reader friendly output (flat text, no decorative borders or animations).

**All three forms are equivalent.** The v2.1.208 release note introduces them together: *"Run `claude --ax-screen-reader`, set `CLAUDE_AX_SCREEN_READER=1`, or add `"axScreenReader": true` to settings."* Relative precedence among the three is unverified; the collection-wide convention (CLI > env > config) is the expected ordering but is not established for this parameter specifically.

**Later refinements.** v2.1.218 added screen-reader announcements for word- and line-level deletions (`Option+Delete`, `Ctrl+W`, `Cmd+Backspace`, `Ctrl+U`, `Ctrl+K`) and fixed VoiceOver reading "new line" instead of echoing a trailing typed space.

### Verification

```bash
claude --help | grep -A2 -- '--ax-screen-reader'   # → the description above

V=~/.local/share/claude/versions/2.1.220
for k in CLAUDE_AX_SCREEN_READER axScreenReader TOTALLY_FAKE_VAR_XYZ; do
  printf '%-26s %s\n' "$k" "$(grep -ac "$k" "$V")"
done   # → 10, 3, 0 (last is the negative control)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [../version/105_v2_1_208.md](../version/105_v2_1_208.md) | Release introducing the flag |
