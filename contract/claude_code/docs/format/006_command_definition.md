# Format: Command Definition

### Scope

- **Purpose**: Specify the `~/.claude/commands/{name}.md` format — markdown files defining custom slash commands for Claude Code sessions.
- **Responsibility**: Authoritative instance for command definition format — file location, naming convention, markdown structure, invocation pattern.
- **In Scope**: File location pattern, naming convention, markdown content structure, `/{command-name}` invocation.
- **Out of Scope**: Command directory context (→ [`../storage/002_support_directories.md`](../storage/002_support_directories.md)).

### Location

`~/.claude/commands/{command-name}.md`

**Format**: Markdown document (free-form).
**Mutability**: Static — modified only when user updates a command definition.

### Schema

Example (`~/.claude/commands/commit.md`):

```markdown
Act as an expert git assistant. Analyze the repository state, manage the staging area, and execute commits with high-quality, conventional commit messages.

## Instructions

1. Run git status to see changes
2. Run git diff to understand modifications
3. Draft commit message following conventions
4. Execute git commit with message
...
```

**Structure**: Free-form markdown. No required sections. Content is injected as the prompt when the user invokes `/command-name`.

### Invocation

A command defined at `~/.claude/commands/commit.md` is invoked in a Claude Code session as `/commit`. The file content is used as the instruction prompt.

### File Count

48 command definition files observed in reference storage.

**Examples**: `commit.md`, `refactor_extracting.md`, `test_clean.md`, `pr_review.md`

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Formats master index |
| storage | [`../storage/002_support_directories.md`](../storage/002_support_directories.md) | `commands/` directory: size, growth |
| tool | [`../tool/013_skill.md`](../tool/013_skill.md) | Skill tool that invokes these command definitions |
