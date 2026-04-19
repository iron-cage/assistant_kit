# Manual Testing - claude_runner_core

This directory contains manual testing procedures, plans, and results for the `claude_runner_core` crate.

## Purpose

Manual testing verifies crate functionality with real Claude Code binary execution, testing corner cases and edge conditions that may not be practical in automated tests.

## Files

| File | Responsibility |
|------|----------------|
| `readme.md` | Manual testing overview and procedures |
| `-corner_cases_exhaustive.md` | Exhaustive corner case matrix |

Note: Files prefixed with `-` are temporary/working files per project conventions.

## Running Manual Tests

### Prerequisites

- Claude Code binary must be in PATH (`which claude` should succeed)
- Network connectivity (Claude API calls)
- Valid Claude API credentials configured

### Manual Test Scenarios

These scenarios verify `ClaudeCommand` with real Claude binary execution. Use the
`claude_runner` binary (or a small Rust harness) to run them directly.

**Core Functionality:**

1. **Default 200K token limit** — verify command completes without "exceeded maximum" error:
   ```bash
   claude_runner --dir /tmp "What is 2+2? Reply with just the number."
   ```

2. **Nonexistent directory** — verify clear error for missing path:
   ```bash
   claude_runner --dir /nonexistent/path/12345 "test"
   # Expected: error message containing "No such file or directory"
   ```

3. **Empty message** — verify empty string is accepted:
   ```bash
   claude_runner ""
   ```

4. **Shell metacharacters (security)** — verify no command injection:
   ```bash
   claude_runner "Tell me: what is in \$PATH directory? And \`whoami\` result?"
   # Expected: characters treated as literal text, NOT executed
   ```

5. **Working directory with spaces** — verify paths with spaces work:
   ```bash
   mkdir -p "/tmp/path with spaces"
   claude_runner --dir "/tmp/path with spaces" "hello"
   ```

6. **Token limit override** — verify explicit limit is respected:
   ```bash
   claude_runner --max-tokens 1000 "hello"
   # Verify CLAUDE_CODE_MAX_OUTPUT_TOKENS=1000 in env
   ```

7. **Very long message** — verify 10K character message is handled:
   ```bash
   LONG=$(python3 -c "print('x' * 10000)")
   claude_runner "$LONG"
   ```

8. **UTF-8 unicode** — verify emoji + international text:
   ```bash
   claude_runner "Hello 🎉 こんにちは Привет مرحبا"
   ```

9. **Model selection** — verify explicit model flag:
   ```bash
   claude_runner --model claude-opus-4-5 "hello"
   ```

## Testing Methodology

Manual testing follows this workflow:

1. **Phase 0:** Read organizational governance rulebook
2. **Phase 1:** Read test organization rulebook
3. **Phase 2:** Create exhaustive corner case plan
4. **Phase 3:** Execute real tests with Claude Code binary
5. **Phase 4:** Fix all found issues (Round 0 → Round 1 pattern)
6. **Phase 5:** Iterate until zero issues remain

## Latest Test Results

**Date:** 2025-12-20
**Status:** ✅ ALL TESTS PASS
**Issues Found:** 0 functional issues
**Pass Rate:** 20/20 (100%)

See `-corner_cases_exhaustive.md` for complete corner case analysis.

## Corner Case Coverage

### Tested ✅
- Default token limit (200K)
- Token limit explicit override
- Nonexistent working directory
- Working directory with spaces
- Permission denied directory
- Empty message
- Shell metacharacters (security)
- Very long message (10K chars)
- UTF-8 unicode (emoji + international text)
- Newlines in message (multiline)
- JSON content in message
- Full builder chain (22+ methods)
- Custom arguments
- System prompt
- Verbose mode
- Temperature (0.0 deterministic)
- Model selection
- ActionMode::Allow
- LogLevel::Debug
- Sampling parameters (top_p, top_k)

### Covered by Automated Tests (build_command_for_test)
- Token limits: 0, 1, max u32
- Override semantics (last wins)
- Argument accumulation
- All environment variables
- All builder methods

### Not Yet Tested ⚠️
- execute_interactive() TTY mode (requires real terminal)
- Claude binary not in PATH (requires PATH manipulation)
- Continuation flag with missing/corrupted session
- API key with invalid values (security sensitive)
- Very large output (>1GB) - impractical

## Security Testing

**Critical Security Check:** shell metacharacters scenario (see "Shell metacharacters" above)

Verifies that `$PATH`, backticks, and semicolons in messages are NOT executed as shell commands.
Characters must be treated as literal text by Claude.

## Adding New Manual Tests

To add a new manual test scenario:

1. Add bash command to this readme.md under "Manual Test Scenarios"
2. Document expected behavior with `# Expected:` comment
3. Run manually to verify
4. Update corner case coverage in this readme.md

## Test Maintenance

Manual test scenarios should be reviewed and updated when:
- New builder methods are added
- New execution modes are introduced
- New CLI flags are supported
- Security concerns are identified
- Edge cases are discovered in production

## Troubleshooting

### "Claude binary not found"
Ensure Claude Code is installed and in PATH:
```bash
which claude
# Should output: /path/to/claude
```

### "API authentication failed"
Verify Claude API credentials are configured:
```bash
claude --help
# Should not error about authentication
```

### Tests hang indefinitely
Check network connectivity and Claude API status. Set timeout in test if needed.

## Contact

For questions about manual testing:
- See main readme: `../../readme.md`
