# Test: Invariant 002 — Cross-Platform Compatibility

Property assertion cases for `docs/invariant/002_cross_platform.md`. Verifies that all path
operations use platform-safe APIs and that home directory resolution never uses a tilde literal.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IN-1 | Source tree contains no path string concatenation | Invariant holds (normal) |
| IN-2 | Home directory resolution uses env vars, not tilde literal | Invariant holds (boundary) |

**Total:** 2 IN cases

---

### IN-1: Source tree contains no path string concatenation

- **Given:** The `src/` tree at the current HEAD
- **When:** `grep -rn 'format!.*".*/"' src/` or equivalent scan for path string concatenation is
  run
- **Then:** No occurrences of path components assembled via string formatting or `+` operator are
  found; all path construction uses `PathBuf::from(…).join(…)` or equivalent `std::path` APIs
- **Source:** [docs/invariant/002_cross_platform.md](../../../docs/invariant/002_cross_platform.md)

---

### IN-2: Home directory resolution uses env vars, not tilde literal

- **Given:** The `src/` tree at the current HEAD
- **When:** `grep -rn '"~/' src/` is run to find any tilde-prefixed literal paths
- **Then:** Zero occurrences are found — all home directory lookups read from `$HOME`
  (Linux/macOS) or `$USERPROFILE` (Windows) env vars via `std::env::var`, never constructing
  paths from the `~` shorthand
- **Source:** [docs/invariant/002_cross_platform.md](../../../docs/invariant/002_cross_platform.md)
