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

- **Given:** an encoded key `k` = `-home-alice-projects-my-app` where the corresponding directory exists on disk
- **When:** `encode_path(decode_path_via_fs(storage_root, k))` is called
- **Then:** the result equals `k` (round-trip holds)

---

### IN-5: Two collision paths disambiguated by filesystem DFS

- **Given:** two project directories exist on disk:
  - `/home/alice/projects/my-app` (with hyphen)
  - `/home/alice/projects/my_app` (with underscore)
  both encoding to `-home-alice-projects-my-app`
- **When:** `decode_path_via_fs(storage_root, "-home-alice-projects-my-app")` is called with the first directory present
- **Then:** returns the filesystem-confirmed path (whichever exists), not an error
