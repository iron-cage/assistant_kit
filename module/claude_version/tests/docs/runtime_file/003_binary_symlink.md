# Runtime File Test: Binary Symlink

### Scope

- **Purpose**: RF- test cases for the binary symlink runtime file — path correctness, retarget lifecycle, and durability.
- **Responsibility**: Verify the symlink path spec, retarget behavior on install/hot-swap, and recoverable durability classification.
- **In Scope**: Path format, HOME expansion, retarget via `.version.install`, durability after deletion.
- **Out of Scope**: Versions directory content and lock state (→ `002_versions_directory.md`), auto-updater retarget bypass vector (→ `../pitfall/002_symlink_retarget.md`).

Runtime file test surface for the binary symlink. See [runtime_file/003_binary_symlink.md](../../../docs/runtime_file/003_binary_symlink.md) for specification.

## Behavioral Divergence Pair

Two states of the symlink that produce different `.status` outcomes:

- **Input A:** Symlink present, pointing at an installed version → `.status`/`.version.show` report that version
- **Input B:** Symlink missing → version commands report "not found" until the next install/hot-swap recreates it

## Test Case Index

| RF | Scenario | Source fn |
|----|----------|-----------|
| RF-1 | Path matches `$HOME/.local/bin/claude` exactly | ⏳ |
| RF-2 | `.version.install` retargets the symlink to the newly installed version's binary | ⏳ |
| RF-3 | Version commands succeed after the symlink is manually deleted (durability classification) | ⏳ |

## Test Coverage Summary

- Path correctness: 1 test (RF-1)
- Lifecycle retarget: 1 test (RF-2)
- Durability: 1 test (RF-3)

**Total:** 3 tests

---

### RF-1: path matches spec — $HOME expansion

- **Given:** `HOME=/tmp/rf_test_home`
- **When:** `.paths key::binary_symlink` output is examined
- **Then:** stdout contains exactly `/tmp/rf_test_home/.local/bin/claude`; path begins with HOME value; path ends with `.local/bin/claude`
- **Exit:** 0
- **Source:** [runtime_file/003_binary_symlink.md — Path](../../../docs/runtime_file/003_binary_symlink.md)

---

### RF-2: install retargets symlink to new version

- **Given:** `HOME=/tmp/rf_test_home`; symlink currently points at an older installed version
- **When:** `clv .version.install version::stable`
- **Then:** exit 0; symlink target now resolves inside the newly installed version's subdirectory under the versions directory
- **Exit:** 0
- **Source:** [runtime_file/003_binary_symlink.md — Lifecycle: Created/retargeted](../../../docs/runtime_file/003_binary_symlink.md)

---

### RF-3: durability — deletion is recoverable, next install re-creates

- **Given:** `HOME=/tmp/rf_test_home`; symlink exists; symlink is then manually deleted
- **When:** `clv .version.install version::stable` is called after deletion
- **Then:** exit 0; command succeeds despite the missing symlink; symlink is re-created pointing at the target version after the call
- **Exit:** 0
- **Source:** [runtime_file/003_binary_symlink.md — Durability](../../../docs/runtime_file/003_binary_symlink.md)

---

## Source Functions

| Function | File | Test Cases |
|----------|------|------------|
| *(not yet implemented)* | `tests/cli/binary_symlink_test.rs` | RF-1 through RF-3 |
