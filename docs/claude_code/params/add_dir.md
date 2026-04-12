# add_dir

Grants tool access to additional directories beyond the current working directory.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--add-dir <dirs...>` |
| Env Var | — |
| Config Key | — |

### Type

path[] (space or comma separated)

### Default

—

### Description

Grants tool access to additional directories beyond the current working directory. Claude's file-reading and editing tools are normally scoped to the project directory; `--add-dir` expands that scope to the listed paths. Can be specified multiple times or as a space-separated list. Does not restrict the base project directory.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |