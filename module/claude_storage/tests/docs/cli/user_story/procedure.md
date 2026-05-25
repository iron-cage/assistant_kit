# User Story Test Execution Procedure

## Trigger

When verifying acceptance criteria for any `docs/cli/user_story/*.md` story.

## Steps

1. Set up an isolated test fixture:
   ```bash
   export CLAUDE_STORAGE_ROOT=/tmp/clg-test-$(date +%s)
   mkdir -p "$CLAUDE_STORAGE_ROOT/projects"
   ```
2. Populate the fixture with sample projects and sessions as required per test case (see each RWS-N for fixture specification).
3. For each RWS-N scenario in order: execute the command, inspect stdout/stderr, verify exit code.
4. Record pass/fail per RWS-N ID in the test run log.
5. Tear down the fixture:
   ```bash
   rm -rf "$CLAUDE_STORAGE_ROOT"
   ```

## Fixture Structure

Standard test fixture layout (minimal):
```
/tmp/clg-test-{ts}/
  projects/
    -home-user-project-a/      ← path-encoded project
      -default_topic.jsonl     ← session file with entries
    8d795a1c-c81d-4010-8d29/   ← UUID-named project
      -default_topic.jsonl
```

## Acceptance Criteria Mapping

Each RWS-N references one or more Acceptance Criteria IDs from the source user story. A test PASSES when:
- stdout matches the expected pattern (checked with `grep` or exact match)
- exit code matches the expected value

## Environment Variables

| Variable | Value | Purpose |
|----------|-------|---------|
| `CLAUDE_STORAGE_ROOT` | `/tmp/clg-test-{ts}` | Isolates tests from real storage |
