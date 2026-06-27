# Tool: Edit

Patch existing files via exact string replacement.

### Category

File Operations

### Description

Performs exact string replacements in files. Requires the file to have been read first. The `old_string` must be unique in the file (or `replace_all` must be set). Preserves exact indentation. Preferred over Write for modifying existing files as it sends only the diff.

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [001_read.md](001_read.md) | Read — required before editing (file must be read first) |
| doc | [002_write.md](002_write.md) | Write — full overwrite alternative (use when Edit is insufficient) |
