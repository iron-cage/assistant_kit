# Decision: Print Mode Requires Content

**ID:** D3 · **Category:** Behavior · **Status:** ✅ Adopted

### Scope

- **Purpose**: Record why explicitly-requested print mode is rejected up front when nothing has been supplied to send.
- **Responsibility**: Rationale for the fail-fast content guard, the BUG-427 revision that widened what counts as content, and the boundary that separates this guard from D11's mode-*selection* formula.
- **In Scope**: What counts as content (message, `--file`, non-empty piped stdin); which requests trigger the guard; why a silent no-op was rejected.
- **Out of Scope**: How print mode gets selected in the first place (→ [011_print_by_default.md](011_print_by_default.md)); `-p`/`--print` reference semantics (→ [`../cli/param/002_print.md`](../cli/param/002_print.md)).

### Decision

Fail fast with a clear error if print mode is *requested* — via `-p`/`--print`, `CLR_PRINT`, or JSON config `"print"` — with no content to send: no message, no `--file`, and no non-empty piped stdin content.

### Rationale

A silent no-op would be confusing; `claude` in print mode without input produces nothing useful. The failure is cheap to detect before the subprocess is spawned, and reporting it costs the user one clear error instead of an empty result they have to diagnose.

### Scope Boundary

This check fires only when `cli.print_mode` is true — set by the explicit `-p`/`--print` flag, `CLR_PRINT`, or JSON config, any of which settle the mode-selection question outright. It validates content once print mode is *already selected* that way, and is orthogonal to [D11](011_print_by_default.md)'s mode-*selection* formula (message / non-TTY stdin / `--file` / stdin content, any one of which routes to print mode without `cli.print_mode` being set at all).

A bare invocation that reaches print mode implicitly through non-TTY stdin alone — no message, no `--file`, no stdin content — does not hit this guard. It proceeds with no content, because `cli.print_mode` was never set true and print mode was reached by inference instead.

### History

**Fixed (BUG-427):** the guard originally keyed on message-absence alone, so it incorrectly rejected `--print --file <path>` even though `--file` supplies the prompt. It now also accepts `--file` or non-empty stdin content as a valid substitute for a message.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Decision collection index |
| decision | [011_print_by_default.md](011_print_by_default.md) | Mode *selection* — the adjacent question this guard does not answer |
| cli | [`../cli/param/002_print.md`](../cli/param/002_print.md) | `-p`/`--print` parameter reference |
| test | `../../tests/execution_mode_ext_test.rs` | Covers the implicit-print path that deliberately bypasses this guard |
