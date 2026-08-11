# Test: `set::` Parameter — RETIRED (Feature 035)

> **RETIRED (Feature 035)**: The `set::` parameter on `.model` has been replaced by the
> scope-relative `model::` parameter, which also absorbed `id::`'s model-pinning role from
> the retired `.model.select` command.
>
> Current CLI: `clp .model model::opus` (session scope, default) or
> `clp .model scope::subprocess model::claude-opus-4-8` (subprocess scope).
>
> See [param/076_model_value.md](../../../../docs/cli/param/076_model_value.md) for the current specification.
> See [feature/035_model_command.md](../../../../docs/feature/035_model_command.md) for the unified `.model` design.

All EC test cases in this file (EC-1 through EC-6) are **superseded** — `set::` no longer exists as a
parameter on `.model`. The equivalent session-scope shorthand behavior (EC-2 through EC-6) is now
exercised by `tests/cli/model_test.rs`: `t04_set_model_session_each_shorthand` (opus/sonnet/haiku),
`t05_set_model_session_default_removes_key` (default), and `t06_set_model_session_invalid_exits_1` (bad value).
Note the current interface also prints a confirmation line on write (e.g. `model: opus  →  <path> (session)`)
rather than writing silently as EC-2 through EC-4 describe below.

### Superseded Test Case Index (DO NOT IMPLEMENT)

| ID | Test Name | Category | Status |
|----|-----------|----------|--------|
| EC-1 | `set::` absent — `.model` operates in get mode; reads and prints model | Behavioral Divergence | **REMOVED** |
| EC-2 | `set::opus` present — `.model` operates in set mode; writes `claude-opus-4-8` | Behavioral Divergence | **REMOVED** |
| EC-3 | `set::sonnet` accepted; writes `claude-sonnet-5` to `settings.json` | Valid Value | **REMOVED** |
| EC-4 | `set::haiku` accepted; writes `claude-haiku-4-5-20251001` to `settings.json` | Valid Value | **REMOVED** |
| EC-5 | `set::default` removes `model` key; other `settings.json` keys preserved | Valid Value | **REMOVED** |
| EC-6 | `set::bad` exits 1; stderr names all four valid values | Invalid Value | **REMOVED** |

---

### EC-1: `set::` absent — `.model` in get mode *(SUPERSEDED)*

- **Given:** `~/.claude/settings.json` contains `{"model": "sonnet"}`.
- **When:** `clp .model` (no `set::` argument)
- **Then:** Exits 0. Stdout is `model: sonnet\n`. No write to `settings.json`.
- **Exit:** 0
- **Source fn:** *(no direct successor — get mode with no action parameter is covered incidentally, not by a dedicated test in `model_test.rs`)*
- **Source:** [param/055_set.md](../../../../docs/cli/param/055_set.md)

---

### EC-2: `set::opus` present — `.model` in set mode *(SUPERSEDED)*

- **Given:** `~/.claude/settings.json` exists (any state).
- **When:** `clp .model set::opus`
- **Then:** Exits 0. `~/.claude/settings.json` contains `"model": "claude-opus-4-8"`. No model text printed to stdout.
- **Exit:** 0
- **Source fn:** `t04_set_model_session_each_shorthand` (in `tests/cli/model_test.rs`) — current CLI is `model::opus`, not `set::opus`; the current interface ALSO prints a confirmation line on write (`model: opus  →  <path> (session)`), unlike this case's "no model text printed" claim
- **Source:** [param/076_model_value.md](../../../../docs/cli/param/076_model_value.md)

---

### EC-3: `set::sonnet` accepted; writes `claude-sonnet-5` *(SUPERSEDED)*

- **Given:** Any `settings.json` state.
- **When:** `clp .model set::sonnet`
- **Then:** Exits 0. `~/.claude/settings.json` contains `"model": "claude-sonnet-5"`.
- **Exit:** 0
- **Source fn:** `t04_set_model_session_each_shorthand` (in `tests/cli/model_test.rs`) — current CLI is `model::sonnet`, not `set::sonnet`
- **Source:** [param/076_model_value.md](../../../../docs/cli/param/076_model_value.md)

---

### EC-4: `set::haiku` accepted; writes `claude-haiku-4-5-20251001` *(SUPERSEDED)*

- **Given:** Any `settings.json` state.
- **When:** `clp .model set::haiku`
- **Then:** Exits 0. `~/.claude/settings.json` contains `"model": "claude-haiku-4-5-20251001"`.
- **Exit:** 0
- **Source fn:** `t04_set_model_session_each_shorthand` (in `tests/cli/model_test.rs`) — current CLI is `model::haiku`, not `set::haiku`
- **Source:** [param/076_model_value.md](../../../../docs/cli/param/076_model_value.md)

---

### EC-5: `set::default` removes model key; other keys preserved *(SUPERSEDED)*

- **Given:** `~/.claude/settings.json` contains `{"model": "claude-opus-4-8", "theme": "dark"}`.
- **When:** `clp .model set::default`
- **Then:** Exits 0. `settings.json` no longer contains `"model"` key. `"theme": "dark"` is preserved.
- **Exit:** 0
- **Source fn:** `t05_set_model_session_default_removes_key` (in `tests/cli/model_test.rs`) — current CLI is `model::default`, not `set::default`
- **Source:** [param/076_model_value.md](../../../../docs/cli/param/076_model_value.md)

---

### EC-6: `set::bad` exits 1; all valid values named in stderr *(SUPERSEDED)*

- **Given:** Any environment (no credential store required).
- **When:** `clp .model set::bad`
- **Then:** Exits 1. Stderr contains each of: `opus`, `sonnet`, `haiku`, `default`.
- **Exit:** 1
- **Source fn:** `t06_set_model_session_invalid_exits_1` (in `tests/cli/model_test.rs`) — current CLI is `model::bad`, not `set::bad`
- **Source:** [param/076_model_value.md](../../../../docs/cli/param/076_model_value.md)
