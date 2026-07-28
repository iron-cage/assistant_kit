# Parameter 066: `reset::` — Edge Cases

**Behavioral Divergence Pair:** EC-01 ↔ EC-02 — `reset::1` removes the `model` key from `~/.clr/config.toml`'s user tier — an observable file-state change reverting clr subprocess selection to `ISOLATED_DEFAULT_MODEL`; `reset::0` (default) is a no-op for reset — mode on `.model.select` is instead determined purely by `id::` presence, and no key is touched.

### Test Case Index

| ID | Test | Scenario | Expected | Status |
|----|------|----------|----------|--------|
| EC-01 | `ec1_reset_1_removes_subprocess_model_key` | `~/.clr/config.toml` has `model = "claude-opus-4-8"` → `reset::1` | key removed from file; exit 0 | ✅ |
| EC-02 | `ec2_reset_0_default_is_noop` | `reset::0` (default), no `id::` | no file change; mode is get (determined by `id::` absence, not by `reset::0`) | ✅ |
| EC-03 | `ec3_reset_omitted_defaults_to_0` | `.model.select` with no `reset::` | behaves identically to `reset::0` | ✅ |
| EC-04 | `ec4_reset_1_and_id_mutually_exclusive` | `reset::1 id::claude-opus-4-8` | exit 1 — `id:: and reset::1 are mutually exclusive` | ✅ |
| EC-05 | `ec5_reset_1_idempotent_when_no_preference_set` | `~/.clr/config.toml` exists but has no `model` key → `reset::1` | exits 0 without error; idempotent no-op | ✅ |
| EC-06 | `ec6_reset_1_idempotent_when_prefs_file_absent` | `~/.clr/config.toml` does not exist → `reset::1` | exits 0 without error; no file created | ✅ |
| EC-07 | `ec7_reset_1_preserves_other_prefs_keys` | `~/.clr/config.toml` has `model` plus unrelated keys → `reset::1` | `model` removed; other keys preserved; exit 0 | ✅ |
| EC-08 | `ec8_reset_true_false_aliases_accepted` | `reset::true` and `reset::false` | `true` behaves as `1` (removes key), `false` behaves as `0` (no-op) | ✅ |
| EC-09 | `ec9_reset_invalid_value_exits_1` | `reset::maybe` (non-boolean) | exit 1 — invalid boolean value rejected | ✅ |
| EC-10 | `ec10_reset_1_stdout_message` | `~/.clr/config.toml` has `model = "claude-sonnet-5"` → `reset::1` | stdout contains `model.select: (reset to default)`; exit 0 | ✅ |

**Total:** 10 edge case tests

---

### EC-01: `reset::1` removes `model` — observable file-state change

- **Given:** `~/.clr/config.toml` contains `model = "claude-opus-4-8"`
- **When:** `clp .model.select reset::1`
- **Then:** Exits 0. `~/.clr/config.toml` no longer contains the `model` key. `clr run/ask/isolated/refresh` subsequently uses `ISOLATED_DEFAULT_MODEL`.
- **Exit:** 0
- **Source:** [param/066_reset.md](../../../../docs/cli/param/066_reset.md), [command/007_model.md](../../../../docs/cli/command/007_model.md)

---

### EC-02: `reset::0` (default) — no-op, mode determined by `id::`

- **Given:** `~/.clr/config.toml` contains `model = "claude-opus-4-8"`
- **When:** `clp .model.select reset::0` (no `id::`)
- **Then:** Exits 0. `~/.clr/config.toml` is unchanged — still contains `model = "claude-opus-4-8"`. Mode is get, determined by `id::` absence, not by the `reset::0` value itself.
- **Exit:** 0
- **Source:** [param/066_reset.md](../../../../docs/cli/param/066_reset.md)

---

### EC-03: Omitted `reset::` defaults to `0`

- **Given:** `~/.clr/config.toml` contains `model = "claude-opus-4-8"`
- **When:** `clp .model.select` (no `reset::` provided, no `id::`)
- **Then:** Exits 0. Behavior is identical to `reset::0` — get mode, file unchanged, same as EC-02.
- **Exit:** 0
- **Source:** [param/066_reset.md](../../../../docs/cli/param/066_reset.md)

---

### EC-04: `reset::1 id::VALUE` together — mutually exclusive, rejected

- **Given:** Any `~/.clr/config.toml` state
- **When:** `clp .model.select reset::1 id::claude-opus-4-8`
- **Then:** Exits 1. stderr contains `id:: and reset::1 are mutually exclusive`. `~/.clr/config.toml` is unchanged.
- **Exit:** 1
- **Source:** [param/066_reset.md](../../../../docs/cli/param/066_reset.md), [param/064_id.md](../../../../docs/cli/param/064_id.md)

---

### EC-05: `reset::1` idempotent when no preference is currently set

- **Given:** `~/.clr/config.toml` exists and contains `other_key = "value"` — no `model` key present.
- **When:** `clp .model.select reset::1`
- **Then:** Exits 0 without error. `~/.clr/config.toml` still contains `other_key = "value"`, unchanged. Operation is idempotent.
- **Exit:** 0
- **Source:** [param/066_reset.md](../../../../docs/cli/param/066_reset.md)

---

### EC-06: `reset::1` idempotent when `config.toml` file is absent

- **Given:** `~/.clr/config.toml` does not exist on disk.
- **When:** `clp .model.select reset::1`
- **Then:** Exits 0 without error. No file is created. stdout contains `model.select: (reset to default)`.
- **Exit:** 0
- **Source:** [param/066_reset.md](../../../../docs/cli/param/066_reset.md), [command/007_model.md](../../../../docs/cli/command/007_model.md)

---

### EC-07: `reset::1` preserves other keys in `config.toml`

- **Given:** `~/.clr/config.toml` contains `model = "claude-opus-4-8"` and `other_key = "unrelated_value"`
- **When:** `clp .model.select reset::1`
- **Then:** Exits 0. `~/.clr/config.toml` contains `other_key = "unrelated_value"` only — `model` removed, all other keys preserved verbatim.
- **Exit:** 0
- **Source:** [param/066_reset.md](../../../../docs/cli/param/066_reset.md)

---

### EC-08: `true`/`false` boolean aliases accepted

- **Given:** `~/.clr/config.toml` contains `model = "claude-opus-4-8"`
- **When:**
  1. `clp .model.select reset::true`
  2. `clp .model.select reset::false` (on a fresh copy with the same starting state)
- **Then:** 1. Behaves identically to `reset::1` — key removed, exit 0.
  2. Behaves identically to `reset::0` — no change, exit 0.
- **Exit:** 0
- **Source:** [param/066_reset.md](../../../../docs/cli/param/066_reset.md)

---

### EC-09: Invalid boolean value rejected

- **Given:** Any `~/.clr/config.toml` state.
- **When:** `clp .model.select reset::maybe`
- **Then:** Exits 1. stderr indicates `maybe` is not a valid boolean value for `reset::`.
- **Exit:** 1
- **Source:** [param/066_reset.md](../../../../docs/cli/param/066_reset.md)

---

### EC-10: `reset::1` stdout confirmation message

- **Given:** `~/.clr/config.toml` contains `model = "claude-sonnet-5"`
- **When:** `clp .model.select reset::1`
- **Then:** Exits 0. stdout contains `model.select: (reset to default)`.
- **Exit:** 0
- **Source:** [param/066_reset.md](../../../../docs/cli/param/066_reset.md), [command/007_model.md](../../../../docs/cli/command/007_model.md)
