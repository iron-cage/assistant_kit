# Runtime File Test: Version Markers

### Scope

- **Purpose**: RF- test cases for the version-markers.json runtime file — path correctness, lifecycle triggers, and durability.
- **Responsibility**: Verify the markers file path spec, creation behavior, graceful degradation on absence or malformed content, and safe-to-lose durability for the file-system level.
- **In Scope**: Path format, HOME expansion, creation via `.version.mark`, graceful degradation when absent or malformed, durability after deletion.
- **Out of Scope**: Marker name/value validation (→ `../cli/command/17_version_mark.md`), resolution behavior in install/guard (→ `../feature/010_custom_markers.md`).

Runtime file test surface for version-markers.json. See [runtime_file/004_version_markers.md](../../../docs/runtime_file/004_version_markers.md) for specification.

## Behavioral Divergence Pair

Two `.version.mark` invocations that produce different file-system outcomes:

- **Input A:** `.version.mark name::my-pin version::2.1.220` with no markers file → file created at `$HOME/.claude/version-markers.json`
- **Input B:** `.version.mark name::my-pin unset::1` with `my-pin` absent → file unchanged (no-op write); both exit 0

## Test Case Index

| RF | Scenario | Source fn |
|----|----------|-----------|
| RF-1 | Path matches `$HOME/.claude/version-markers.json` exactly | ⏳ |
| RF-2 | `.version.mark name::N version::V` creates markers file on first invocation when absent | ⏳ |
| RF-3 | `.version.list` succeeds after markers file is manually deleted (durability: absence is safe) | ⏳ |
| RF-4 | `.version.list` succeeds when markers file contains invalid JSON (graceful degradation) | ⏳ |

## Test Coverage Summary

- Path correctness: 1 test (RF-1)
- Lifecycle creation: 1 test (RF-2)
- Durability / absent: 1 test (RF-3)
- Graceful degradation: 1 test (RF-4)

**Total:** 4 tests

---

### RF-1: path matches spec — $HOME expansion

- **Given:** `HOME=/tmp/rf_test_home`
- **When:** `.version.mark name::my-pin version::2.1.220` is invoked
- **Then:** `~/.claude/version-markers.json` resolves to `/tmp/rf_test_home/.claude/version-markers.json`; the file is created at that exact path
- **Exit:** 0
- **Source:** [runtime_file/004_version_markers.md — Path](../../../docs/runtime_file/004_version_markers.md)

---

### RF-2: file created by first .version.mark set call when absent

- **Given:** `HOME=/tmp/rf_test_home` where `version-markers.json` does NOT exist
- **When:** `clv .version.mark name::my-pin version::2.1.220`
- **Then:** exit 0; `$HOME/.claude/version-markers.json` exists on disk after the call; file contains a JSON object with a `"markers"` array
- **Exit:** 0
- **Source:** [runtime_file/004_version_markers.md — Lifecycle: Created](../../../docs/runtime_file/004_version_markers.md)

---

### RF-3: durability — absence is safe, .version.list still exits 0

- **Given:** `HOME=/tmp/rf_test_home`; no `version-markers.json` present
- **When:** `clv .version.list`
- **Then:** exit 0; command succeeds; built-in aliases (`stable`, `latest`) appear in output; no crash
- **Exit:** 0
- **Source:** [runtime_file/004_version_markers.md — Durability](../../../docs/runtime_file/004_version_markers.md)

---

### RF-4: graceful degradation — malformed JSON treated as empty marker set

- **Given:** `HOME=/tmp/rf_test_home`; `version-markers.json` contains invalid JSON (`"not valid json {{{"`)
- **When:** `clv .version.list`
- **Then:** exit 0; built-in aliases appear in output; no custom marker entries shown; no crash
- **Exit:** 0
- **Source:** [runtime_file/004_version_markers.md — Lifecycle: Graceful degradation](../../../docs/runtime_file/004_version_markers.md)

---

## Source Functions

| Function | File | Test Cases |
|----------|------|------------|
| `it18_mark_malformed_json_graceful` | `tests/cli/mutation_version_mark_test.rs` | RF-4 |
| *(not yet implemented)* | `tests/cli/mutation_version_mark_test.rs` | RF-1, RF-2, RF-3 |
