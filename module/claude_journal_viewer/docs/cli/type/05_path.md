# CLI Type: Path

Filesystem path semantic type. Accepts absolute and relative paths.
Tilde (`~`) expansion is performed for home directory references.

- **Kind:** Semantic
- **Fundamental:** String
- **Key Constraint:** Valid filesystem path

### Validation

- Path must be valid UTF-8
- Tilde expansion: `~/...` resolved to `$HOME/...`
- For read paths (`journal_dir`, `dir`): directory must exist
- For write paths (`output`): parent directory must exist

### Error Handling

- Non-existent read path: exit 1 with `Error: journal directory '<path>' does not exist`
- Non-writable write path: exit 1 with `Error: cannot write to '<path>': <io_error>`

### Referenced Parameters

| # | Parameter | Mode |
|---|-----------|------|
| 07 | [`dir`](../param/07_dir.md) | Substring filter (not existence-checked) |
| 21 | [`journal_dir`](../param/21_journal_dir.md) | Read (must exist) |
| 23 | [`output`](../param/23_output.md) | Write (parent must exist) |
