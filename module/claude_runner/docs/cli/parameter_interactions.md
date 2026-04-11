# Parameter Interactions

How `clr` parameters interact when combined. See [params.md](params.md) and [parameter_groups.md](parameter_groups.md) for individual parameter specifications.

## Interaction Table

| Parameters | Interaction | Outcome |
|------------|-------------|---------|
| `--dry-run` + `--trace` | Precedence | `--dry-run` wins; command previewed, execution skipped |
| `--dry-run` + `--verbosity 0` | Independent | `--dry-run` output always shown regardless of verbosity level |
| `[MESSAGE]` (no flag) | Auto-print | Print mode activated automatically when message is present |
| `[MESSAGE]` + `--interactive` | Suppression | `--interactive` suppresses auto-`--print`; TTY passthrough used instead |
| `[MESSAGE]` + `-p`/`--print` | Redundant | Both select print mode; explicit `-p` is a backward-compatible alias |
| `--new-session` (present) | Suppression | `-c` flag omitted from claude invocation; fresh session started |
| `--new-session` (absent) | Default | `-c` injected automatically; previous session continued |
| `--no-skip-permissions` (present) | Suppression | `--dangerously-skip-permissions` not injected into claude invocation |
| `--no-skip-permissions` (absent) | Default | `--dangerously-skip-permissions` injected automatically |
| `--system-prompt` + `--append-system-prompt` | Additive | Both forwarded to claude in parse order; system-prompt replaces, then append adds |
| `--system-prompt` + `[MESSAGE]` | Independent | System prompt sets behavioral context; message sets user turn |
| `--dir` + `--session-dir` | Independent | `--dir` changes working directory; `--session-dir` changes session storage location |
| `--model` + `--dry-run` | Additive | Model appears in dry-run preview; no execution occurs |
| `--verbosity` + `[MESSAGE]` | Independent | Verbosity gates runner diagnostics only; does not affect claude output |

## Mode Conflicts

`--dry-run` and `--trace` both emit command information but differ in execution:

```
clr --dry-run "test"         -- preview only; exits without launching claude
clr --trace "test"           -- prints to stderr, then executes
clr --dry-run --trace "test" -- dry-run wins; no execution
```

`--interactive` and print mode (auto or explicit) are mutually exclusive per invocation:

```
clr "Fix bug"                -- print mode (auto: message present)
clr -p "Fix bug"             -- print mode (explicit alias; same outcome)
clr --interactive "Fix bug"  -- interactive; --print suppressed
clr                          -- interactive REPL (no message; --print never added)
```

## System Prompt Combinations

Both system-prompt flags may be given together:

```sh
clr --system-prompt "You are a Rust expert." \
    --append-system-prompt "Be concise." \
    "Explain this trait"
```

`--system-prompt` replaces the default system prompt entirely.
`--append-system-prompt` adds on top of whatever prompt is active (default or replaced).
When both are given, replacement happens first, then the append is added on top.

### What survives `--system-prompt` replacement

Tool definitions (~12,000 tokens: Bash, Read, Write, Edit, Glob, Grep, WebFetch, etc.)
are injected into the assembled system prompt before the replacement is applied and
survive intact. Claude can still execute all tools normally after replacement.

What is lost: all behavioral guidance — coding guidelines, git safety rules, security
constraints, CLAUDE.md-handling instructions, output style, and sub-agent coordination
prompts. Use `--system-prompt` only when building specialized agents that specify all
of this manually. For adding instructions without losing existing behavior, use
`--append-system-prompt` instead.

| Layer | `--system-prompt` | `--append-system-prompt` | `--tools ""` ❓ |
|-------|------------------|--------------------------|-----------------|
| Tool definitions (~12k tokens) | ✅ Preserved | ✅ Preserved | ❓ Likely preserved (unverified) |
| Tool invocation | ✅ Enabled | ✅ Enabled | ❓ Likely blocked (unverified) |
| Coding guidelines and style | ❌ Removed | ✅ Kept | ✅ Kept |
| Git safety rules | ❌ Removed | ✅ Kept | ✅ Kept |
| Security constraints | ❌ Removed | ✅ Kept | ✅ Kept |
| CLAUDE.md handling instructions | ❌ Removed | ✅ Kept | ✅ Kept |
| Output style (conciseness, no emojis) | ❌ Removed | ✅ Kept | ✅ Kept |
| Sub-agent coordination prompts | ❌ Removed | ✅ Kept | ✅ Kept |

**Hypothesis (H1) — `--tools ""`:** Passing an empty tools list blocks tool *invocation*
but does NOT strip tool *definitions* from the system prompt. The ~12k token cost is paid
regardless — Claude knows about tools but cannot call them. Status: ❓ unverified.
Validation: run `claude --tools "" --print "hi" --output-format json | jq '.usage'` and
compare input token count against a baseline run without `--tools ""`; then observe
whether Claude attempts tool calls in a live conversation.

### `--append-system-prompt` vs CLAUDE.md

These two mechanisms look similar but operate at different layers:

- `--append-system-prompt` → injected into the **system prompt** (highest priority)
- `CLAUDE.md` → injected as the **first user message** (lower priority, different caching)

`--append-system-prompt` instructions have stronger persistence. `CLAUDE.md` instructions
are visible to the model as conversation context rather than system-level directives.
When both are active, `--append-system-prompt` takes precedence for conflicting instructions.

## Precedence Rules

`--dry-run` takes precedence over execution regardless of other flags. If present, no subprocess is launched:

```
clr --dry-run --new-session "test"   -- shows command without -c; does not execute
clr --dry-run --trace "test"         -- dry-run wins over trace; no execution
```

Default injection rules (all are default-on):
- `-c` is injected unless `--new-session` is given
- `--dangerously-skip-permissions` is injected unless `--no-skip-permissions` is given
- `--chrome` is injected via builder default (`ClaudeCommand::new()`); no clr-level suppression flag exists

## Independent Parameters

These parameters operate on orthogonal dimensions and do not interact:

- `--dir` and `--session-dir`: working directory vs. session storage path; both can appear together
- `--model` and `--max-tokens`: model selection vs. token budget; both forwarded independently
- `--verbosity` and any output flag: runner diagnostic level vs. Claude output structure
- `--system-prompt` and `[MESSAGE]`: system turn context vs. user turn content

## Scope

**Applies to all invocations:**
- `--dir`, `--session-dir`, `--max-tokens`, `--verbosity`
- `--no-skip-permissions` (controls automatic `--dangerously-skip-permissions` injection)
- `--new-session` (controls automatic `-c` injection)

**Applies to message-bearing invocations only:**
- `-p`/`--print`, `--interactive`: print vs. TTY passthrough selection
- `--model`, `--verbose`: forwarded to claude subprocess
- `--system-prompt`, `--append-system-prompt`: forwarded to claude subprocess

**Applies to diagnostic/preview invocations:**
- `--dry-run`: preview-only; no subprocess launched
- `--trace`: prints env+command to stderr before launching subprocess
