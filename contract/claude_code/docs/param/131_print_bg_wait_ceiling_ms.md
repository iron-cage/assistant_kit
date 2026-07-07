# print_bg_wait_ceiling_ms

Bounds how long print/headless mode waits for outstanding background tasks before exiting.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS` |
| Config Key | — |

### Type

integer (milliseconds)

### Default

`600000` (10 minutes) — confirmed via the fallback constant in the installed
binary (`... ?? 600000`)

### Since

≤ v2.1.197 (undocumented — not present in any changelog entry 001-098; confirmed
only via string/reference inspection of the installed binary)

### Description

In `-p`/`--print` mode (see [051_print.md](051_print.md)) — where the process is
expected to exit once its main turn completes rather than stay interactive —
this value caps how long the process will keep waiting for outstanding
background shells, agents, or workflows before it finalizes exit anyway.
Decompiled logic (minified, function names not preserved) confirms the
constant is read once as a ceiling and combined with `runningBackgroundTasks`,
`hasActiveTeammates`, `hasPendingNotification`, and a `deadline`/`swept` state
machine to decide, each tick, whether to keep waiting or to sweep (finalize)
the outstanding work and exit.

A sibling project in this same workspace (`claude_storage/.claude/settings.local.json`)
already sets this to `"0"` — i.e. do not wait at all for background work in
print mode, exit immediately. That is a real precedent for headless/CI-style
invocations that would rather abandon outstanding background work than block
the parent process; `claude_runner` itself does not currently set this (it
invokes `claude` as a subprocess and manages its own timeout via
`CLR_TIMEOUT`/`gate_poll_secs`/`gate_max_attempts` instead — see
`module/claude_runner/docs/cli/003_env_param.md`).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [051_print.md](051_print.md) | `-p`/`--print` mode this ceiling applies to |
| doc | [128_bg_tasks_report_running.md](128_bg_tasks_report_running.md) | Related "is a background task still outstanding" signal consumed by the same wait/sweep logic |
| doc | [../../../../module/claude_runner/docs/cli/003_env_param.md](../../../../module/claude_runner/docs/cli/003_env_param.md) | `claude_runner`'s own `CLR_*` timeout/gate vars — the runner-native analogue of this upstream ceiling |
