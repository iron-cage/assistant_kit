# Tool: Read

Read files from the local filesystem.

### Category

File Operations

### Description

Reads a file and returns its content with line numbers. Supports text files, images (PNG, JPG), PDFs (with page range selection), and Jupyter notebooks (.ipynb). Cannot read directories. Lines longer than 2000 characters are truncated. Default limit is 2000 lines from file start; offset and limit parameters allow partial reads.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
