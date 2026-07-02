# Invariant :: Path Encoding

Direct contract tests for the path encode/decode behavioral invariant.

**Source:** [invariant/01_path_encoding.md](../../../docs/invariant/01_path_encoding.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | `/` encodes to `-` | Encoding Rule |
| IN-2 | `_` encodes to `-` (lossy) | Encoding Rule |
| IN-3 | Letters, digits, hyphens pass through unchanged | Encoding Rule |
| IN-4 | `encode(decode(k)) == k` round-trip | Round-Trip |
| IN-5 | Two collision paths disambiguated by filesystem DFS | Disambiguation |

## Test Coverage Summary

- Encoding Rule: 3 tests (IN-1, IN-2, IN-3)
- Round-Trip: 1 test (IN-4)
- Disambiguation: 1 test (IN-5)

**Total:** 5 invariant contract cases

**Implementation target:** `tests/invariant_contracts_test.rs`

## Test Cases

---

### IN-1: `/` encodes to `-`

- **Given:** path `/home/alice/projects/my-app`
- **When:** `encode_path("/home/alice/projects/my-app")`
- **Then:** returns `-home-alice-projects-my-app`

---

### IN-2: `_` encodes to `-` (lossy)

- **Given:** path `/home/alice/projects/my_app`
- **When:** `encode_path("/home/alice/projects/my_app")`
- **Then:** returns `-home-alice-projects-my-app` (identical to IN-1's encoded form — encoding is lossy)

---

### IN-3: Letters, digits, hyphens pass through unchanged

- **Given:** encoded key `-home-alice-projects-my-app`
- **When:** character-level inspection of encode output
- **Then:** no character outside `[a-zA-Z0-9-]` appears in the result for a typical filesystem path

---

### IN-4: `encode(decode(k)) == k` round-trip

- **Given:** a project path containing an underscore component (e.g., `/tmp/my_proj`) where the directory exists on disk and a session has been written to the storage directory keyed by `encode_path(path)`
- **When:** `.projects scope::global` is run with `CLAUDE_STORAGE_ROOT` set to a temp storage root
- **Then:** the display path in the output contains the original path string including the underscore component (round-trip holds: the filesystem-guided decode restores `_` correctly, not `/`)

---

### IN-5: Two collision paths disambiguated by filesystem DFS

- **Given:** two directories encoding to the same storage key (e.g., `/tmp/my-app` with hyphen and `/tmp/my_app` with underscore both encoding to `-tmp-my-app`); a session written under that storage key; at least one of the two directories present on disk
- **When:** `.projects scope::global` is run with `CLAUDE_STORAGE_ROOT` set to a temp storage root
- **Then:** the session display path matches a real filesystem candidate (not a garbled path); when neither candidate directory exists on disk, the raw encoded key is shown rather than a silently corrupted path
