# Subcommand: ultrareview

Run a cloud-hosted multi-agent code review of the current branch (or a PR number / base branch) and print the findings.

### Usage

```
claude ultrareview [options] [target]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `[target]` | A PR number or base branch. Omit to review the current branch. |

### Options

| Flag | Description |
|------|-------------|
| `--json` | Print the raw `bugs.json` payload instead of formatted findings |
| `--timeout <minutes>` | Maximum minutes to wait for the review to finish (default: 30) |
| `-h`, `--help` | Display help for command |

### Sub-subcommands

None.

### Description

Non-interactive entry point to the same cloud multi-agent review that the
`/ultrareview` slash command runs inside a session. Intended for CI and
scripting: findings go to stdout, and the process exits 0 on completion or 1 on
failure.

The review is cloud-hosted and billed. It runs parallel agents that analyse and
critique the diff, then prints the surviving findings.

With no `[target]`, it reviews the current branch — this form bundles the local
branch and does not require a GitHub remote. Passing a PR number instead
fetches and reviews that pull request.

### Since

v2.1.120 for the `claude ultrareview [target]` subcommand. The `/ultrareview`
slash command predates it, introduced in v2.1.111.

### Verification

```bash
claude ultrareview --help      # → Usage: claude ultrareview [options] [target]
claude --help | grep ultrareview
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master subcommand table |
| version | [../version/036_v2_1_120.md](../version/036_v2_1_120.md) | Release that introduced this subcommand |
| version | [../version/028_v2_1_111.md](../version/028_v2_1_111.md) | Release that introduced the `/ultrareview` slash command |
