# from_pr

Resumes a session previously linked to a GitHub pull request.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--from-pr [value]` |
| Env Var | — |
| Config Key | — |

### Type

string? (optional PR number or URL)

### Default

—

### Description

Resumes a session that was previously linked to a GitHub pull request. With no argument, opens an interactive picker. With a PR number or URL, resumes the session associated with that PR directly. Requires the session to have been originally started with PR context. Useful for continuing code review or PR-related work across multiple sessions.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |