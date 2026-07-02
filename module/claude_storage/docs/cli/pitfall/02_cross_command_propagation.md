# Pitfall: Cross-Command Bug Propagation

### Scope

- **Purpose**: Document the cross-command bug propagation pitfall.
- **Responsibility**: Why fixing one command requires grepping all commands for the same pattern.
- **In Scope**: Shared implementation patterns, copy-paste bug amplification.
- **Out of Scope**: Architecture refactoring, deduplication strategies.

### Pitfall

When a bug is found in one command routine, identical logic often exists in multiple other
command routines that were written from the same pattern. Fixing only the discovered instance
leaves the same bug active in all others. Issues #009, #010, and #012 all followed this
propagation pattern — each required fixes in 3–5 command files after the initial discovery.

### Trigger

Fixing a logic bug in (e.g.) `.count` without searching `.show`, `.list`, `.search`,
`.export`, and the session commands for the same flawed pattern.

### Required Pattern

After fixing any logic bug in a command routine:

1. Extract the buggy pattern as a grep query (e.g., the wrong field name, the missing
   validation call, the incorrect default assumption)
2. Search all files in `src/cli/` for the same pattern:
   ```bash
   grep -rn "the_buggy_pattern" src/cli/
   ```
3. Apply the same fix to every match found
4. Add a `// Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.`
   comment at each fix site

This is a mechanical step, not optional. Even if you are confident the bug is isolated,
run the grep.

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| All 11 | *(all commands)* | Any bug found in one command may exist in all others |

### Sources

- `src/cli/search.rs:89,108,129` — repeated propagation-fix comments from issues #009, #012
- `src/cli/count.rs:110,147` — same pattern from issues #010, #012
- `changelog.md` — issues 009, 010, 012 each required multi-file fixes
