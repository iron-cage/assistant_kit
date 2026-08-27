# project_dir_name

Overrides the encoded directory name used for a project's storage.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_PROJECT_DIR_NAME` |
| Config Key | — |

### Type

string (directory name)

### Default

— (unset; the encoded working-directory path is used)

### Since

**Not present in v2.1.220.** Documented officially for a later release (v2.1.234+). The string occurs **0 times** in the installed v2.1.220 binary.

### Description

Replaces the auto-derived project directory name under `<config-dir>/projects/` with an explicit one.

**This entry is a forward reference, not a live parameter.** It is documented here deliberately, and it doubles as the *method control* for this collection's binary string-scan technique: an officially-documented variable that legitimately scans to 0 because the installed binary predates it. That distinguishes "the scan method is broken" from "this variable genuinely isn't in this build" — the distinction that a plain 0 cannot make on its own.

**What it addresses.** The default encoding is lossy and irreversible: every non-alphanumeric character in the absolute path becomes `-`, and paths over 200 characters are truncated with a hash appended. Two different real paths can therefore collide onto one directory name. See [`../behavior/009_b9_storage_path_encoding.md`](../behavior/009_b9_storage_path_encoding.md) for the full rule and the live survey behind it. An explicit name sidesteps both the collision risk and the unreadability.

**Do not write code against it on v2.1.220.** Setting it on this build is a no-op — the storage path is still derived by encoding the working directory.

### Verification

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_PROJECT_DIR_NAME "$V"   # → 0  (absent from THIS build)
grep -ac CLAUDE_CONFIG_DIR            "$V"   # → 28 (positive control: method works)
grep -ac TOTALLY_FAKE_VAR_XYZ         "$V"   # → 0  (negative control)

claude --version                              # → confirm which build you scanned
ls ~/.local/share/claude/versions/            # → re-run the scan on a newer one
```

The positive control is what makes the 0 interpretable: the method finds env vars that *are* present, so the 0 here reflects the binary, not the technique.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [154_config_dir.md](154_config_dir.md) | `CLAUDE_CONFIG_DIR` — relocates the root this name sits under |
| doc | [../behavior/009_b9_storage_path_encoding.md](../behavior/009_b9_storage_path_encoding.md) | The encoding rule this overrides |
| doc | [../storage/readme.md](../storage/readme.md) | `~/.claude/projects/` layout |
