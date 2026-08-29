# Decision: Print By Default

**ID:** D11 · **Category:** Behavior · **Status:** ✅ Adopted

### Scope

- **Purpose**: Record why `clr` routes to print mode by default and reserves the interactive REPL for a genuine terminal with nothing to send.
- **Responsibility**: Rationale for the three print-mode triggers, the `--interactive` opt-in, and the BUG-425/427 widening that added the second and third triggers.
- **In Scope**: The mode-*selection* formula; why automation is treated as the default intent; what bare `clr` still does from a terminal.
- **Out of Scope**: The content guard that fires *after* print mode is explicitly requested (→ [003_print_mode_requires_content.md](003_print_mode_requires_content.md)); flag reference semantics and the `--interactive` / `-p` precedence rule (→ [`../feature/006_cli_design.md`](../feature/006_cli_design.md) § Design).

### Decision

`clr` defaults to print mode — captured stdout via `execute()` + `--print` — when any one of three conditions holds:

1. `[MESSAGE]` is provided
2. stdin is not a terminal (piped, redirected, or a non-interactive shell)
3. `--file` or piped stdin content supplies the prompt

Interactive TTY passthrough requires explicit `--interactive`.

### Rationale

The primary use of `clr "message"` is scripting and automation — piping output, capturing into variables, chaining with other tools. Interactive TTY passthrough is the minority case when a message is given. Defaulting to print mode avoids forcing users to remember `-p` for every scripted invocation and aligns with shell expectations: running a command with an argument should produce capturable output.

A bare invocation under non-TTY stdin (BUG-425) and a `--file`/stdin-content-only invocation (BUG-427) carry the same automation intent as a message argument — none of them is a user sitting at a live terminal — so all three route to print mode on the same basis. The unifying test is not "was a message given" but "is there a human at a terminal waiting to type."

### Consequence

- `clr "Fix bug"` now behaves like `clr -p "Fix bug"` did before
- `-p`/`--print` is kept as a backward-compatible explicit alias
- `--interactive` opts into TTY passthrough when a message is given
- Bare `clr` with no message, invoked from a genuine terminal, still opens the interactive REPL as before
- The same bare invocation under non-TTY stdin, or with `--file`/piped stdin content and no message, now routes to print mode instead (BUG-425/427 fix — see Plan 005)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Decision collection index |
| decision | [003_print_mode_requires_content.md](003_print_mode_requires_content.md) | The content guard that applies once print mode is *explicitly requested* |
| feature | [`../feature/006_cli_design.md`](../feature/006_cli_design.md) | Mode-selection specification, including `--interactive` / `-p` precedence |
| cli | [`../cli/param/002_print.md`](../cli/param/002_print.md) | `-p`/`--print` parameter reference |
| cli | [`../cli/user_story/002_print_mode_capture.md`](../cli/user_story/002_print_mode_capture.md) | End-to-end capture scenario driven by this default |
| invariant | [`../invariant/007_print_mode_timeout.md`](../invariant/007_print_mode_timeout.md) | Print-mode sessions run unlimited unless a timeout is expressed |
