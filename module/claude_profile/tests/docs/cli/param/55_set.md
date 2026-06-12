# Test: `set::` Parameter

Edge case coverage for the `set::` parameter on `.model`. See [param/055_set.md](../../../../docs/cli/param/055_set.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `set::` absent — `.model` operates in get mode; reads and prints model | Behavioral Divergence |
| EC-2 | `set::opus` present — `.model` operates in set mode; writes `claude-opus-4-6` | Behavioral Divergence |
| EC-3 | `set::sonnet` accepted; writes `claude-sonnet-4-6` to `settings.json` | Valid Value |
| EC-4 | `set::haiku` accepted; writes `claude-haiku-4-5-20251001` to `settings.json` | Valid Value |
| EC-5 | `set::default` removes `model` key; other `settings.json` keys preserved | Valid Value |
| EC-6 | `set::bad` exits 1; stderr names all four valid values | Invalid Value |

---

### EC-1: `set::` absent — `.model` in get mode

- **Given:** `~/.claude/settings.json` contains `{"model": "sonnet"}`.
- **When:** `clp .model` (no `set::` argument)
- **Then:** Exits 0. Stdout is `model: sonnet\n`. No write to `settings.json`.
- **Exit:** 0
- **Source fn:** `ec1_set_absent_get_mode_reads_and_prints`
- **Source:** [param/055_set.md](../../../../docs/cli/param/055_set.md)

---

### EC-2: `set::opus` present — `.model` in set mode

- **Given:** `~/.claude/settings.json` exists (any state).
- **When:** `clp .model set::opus`
- **Then:** Exits 0. `~/.claude/settings.json` contains `"model": "claude-opus-4-6"`. No model text printed to stdout.
- **Exit:** 0
- **Source fn:** `ft05_set_opus_writes_full_id`
- **Source:** [param/055_set.md](../../../../docs/cli/param/055_set.md)

---

### EC-3: `set::sonnet` accepted; writes `claude-sonnet-4-6`

- **Given:** Any `settings.json` state.
- **When:** `clp .model set::sonnet`
- **Then:** Exits 0. `~/.claude/settings.json` contains `"model": "claude-sonnet-4-6"`.
- **Exit:** 0
- **Source fn:** `ft06_set_sonnet_writes_full_id`
- **Source:** [param/055_set.md](../../../../docs/cli/param/055_set.md)

---

### EC-4: `set::haiku` accepted; writes `claude-haiku-4-5-20251001`

- **Given:** Any `settings.json` state.
- **When:** `clp .model set::haiku`
- **Then:** Exits 0. `~/.claude/settings.json` contains `"model": "claude-haiku-4-5-20251001"`.
- **Exit:** 0
- **Source fn:** `ft07_set_haiku_writes_full_id`
- **Source:** [param/055_set.md](../../../../docs/cli/param/055_set.md)

---

### EC-5: `set::default` removes model key; other keys preserved

- **Given:** `~/.claude/settings.json` contains `{"model": "claude-opus-4-6", "theme": "dark"}`.
- **When:** `clp .model set::default`
- **Then:** Exits 0. `settings.json` no longer contains `"model"` key. `"theme": "dark"` is preserved.
- **Exit:** 0
- **Source fn:** `ft08_set_default_removes_key_preserves_others`
- **Source:** [param/055_set.md](../../../../docs/cli/param/055_set.md)

---

### EC-6: `set::bad` exits 1; all valid values named in stderr

- **Given:** Any environment (no credential store required).
- **When:** `clp .model set::bad`
- **Then:** Exits 1. Stderr contains each of: `opus`, `sonnet`, `haiku`, `default`.
- **Exit:** 1
- **Source fn:** `ft09_set_bad_value_exits_1`
- **Source:** [param/055_set.md](../../../../docs/cli/param/055_set.md)
