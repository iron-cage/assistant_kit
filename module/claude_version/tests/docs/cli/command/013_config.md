# Test: `.config`

### Scope

- **Purpose**: Integration test cases for the `.config` command.
- **Responsibility**: Test factor analysis, case index, and expected behavior for all four modes (show-all, get, set, unset).
- **In Scope**: Mode dispatch, parameter combinations, exit codes, output format, scope targeting, dry-run, invalid combinations.
- **Out of Scope**: Resolution algorithm unit tests (→ `../../algorithm/002_config_resolution.md`), parameter edge cases (→ `../param/`).

Integration test planning for `.config`. See [command/config.md](../../../../docs/cli/command/config.md) for specification.

## Test Factor Analysis

### Factor 1: Mode (derived from params)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| no params | show-all mode | Default behavior |
| key only | get mode | Read-only |
| key+value | set mode | Write user scope |
| key+value+scope::project | set project scope | Write project scope |
| key+unset::1 | unset mode | Delete key |

### Factor 2: `format::` (String, optional, default text)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Default text output | Default behavior |
| `text` | Explicit text | Explicit valid |
| `json` | JSON output with source fields | Alternate valid |
| `xml` | Unrecognized value | Invalid: exit 1 |

### Factor 3: `dry::` (Boolean, optional, default 0)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent | Real write | Default behavior |
| 1 | Preview only | Explicit true |
| 2 | Out-of-range boolean | Invalid: exit 1 |

### Factor 4: `scope::` (String, optional, default user)

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| absent / `user` | User settings.json | Default behavior |
| `project` | Project settings.json | Alternate valid |
| `global` | Unrecognized value | Invalid: exit 1 |

### Factor 5: Invalid combinations

| Combination | Equivalence Class |
|-------------|-------------------|
| value:: without key:: | Invalid: exit 1 |
| unset::1 without key:: | Invalid: exit 1 |
| value:: and unset::1 together | Invalid: exit 1 |

### Factor 6: HOME environment

| Level | Description | Equivalence Class |
|-------|-------------|-------------------|
| set | Normal | Happy path |
| empty | Cannot resolve path | Failure: exit 2 |

---

## Test Matrix

### Positive Tests

| TC | Description | Mode | Exit | Factors |
|----|-------------|------|------|---------|
| IT-1 | No params → show-all with source labels | show-all | 0 | F1=no-params |
| IT-2 | `key::theme` → get with source annotation | get | 0 | F1=key-only |
| IT-3 | `key::theme value::dark` → set user, bool inferred | set | 0 | F1=key+value |
| IT-4 | `key::model value::claude-opus-4-6 scope::project` → project write | set | 0 | F1=scope-project, F4=project |
| IT-5 | `key::theme unset::1` → key removed from user settings | unset | 0 | F1=unset |
| IT-6 | `format::json` → JSON with source fields | show-all | 0 | F2=json |
| IT-7 | `key::model` with `CLAUDE_MODEL` set → shows env value | get | 0 | F1=key-only |
| IT-8 | `key::unknownArbitraryKey value::v` → accepted, written | set | 0 | F1=key+value |
| IT-9 | `key::model` no env/config → shows catalog default | get | 0 | F1=key-only |
| IT-10 | `key::theme value::dark dry::1` → preview, no write | set | 0 | F3=1 |

### Negative Tests

| TC | Description | Mode | Exit | Factors |
|----|-------------|------|------|---------|
| IT-11 | `value::v` without `key::` → exit 1 | — | 1 | F5=value-without-key |
| IT-12 | `unset::1` without `key::` → exit 1 | — | 1 | F5=unset-without-key |
| IT-13 | `value::v unset::1 key::k` → exit 1 (mutually exclusive) | — | 1 | F5=value+unset |
| IT-14 | `scope::global` → exit 1 (invalid value) | — | 1 | F4=global |
| IT-15 | `format::xml` → exit 1 | — | 1 | F2=xml |
| IT-16 | `HOME` unset → exit 2 | — | 2 | F6=empty |
| IT-17 | `dry::2` → exit 1, out-of-range | — | 1 | F3=2 |

### Summary

- **Total:** 17 tests (10 positive, 7 negative)
- **Negative ratio:** 41.2% ✅ (≥40%)
- **TC range:** IT-1 to IT-17

---

## Coverage Verification

### Exit Status Coverage

| Exit Code | Meaning | Tests |
|-----------|---------|-------|
| 0 | Success | IT-1 through IT-10 |
| 1 | Invalid arguments | IT-11 through IT-15, IT-17 |
| 2 | Runtime error | IT-16 |

### Mode Coverage

| Mode | Tests |
|------|-------|
| show-all | IT-1, IT-6 |
| get | IT-2, IT-7, IT-9 |
| set (user) | IT-3, IT-8, IT-10 |
| set (project) | IT-4 |
| unset | IT-5 |

---

### Source Functions

| Function | File |
|----------|------|
| (all pending) | ⏳ TBD — integration/config_commands_test.rs |
