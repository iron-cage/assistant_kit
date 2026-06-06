# Completed Tasks — claude_runner

### Scope

Completed task files for the `claude_runner` crate. Each file documents a resolved work item with full context.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `010_optional_creds_default.md` | Make --creds optional with $HOME default fallback |
| `012_error_classification.md` | Add ErrorKind enum and classify_error() for labeled diagnostics |
| `013_ask_alias_simplification.md` | Remove ask overrides; make ask a pure alias for run |
| `014_output_file_param.md` | Implement --output-file tee behavior in run_print_mode |
| `015_expect_validation_group.md` | Implement --expect / --expect-strategy / --expect-retries validation |
| `016_bug247_stdout_swallowed.md` | Forward stdout to stderr on non-zero exit (BUG-247) |
| `017_bug248_keep_claudecode_warning.md` | Warn when --keep-claudecode enables nested-agent mode (BUG-248) |
| `018_max_sessions_gate.md` | Implement --max-sessions concurrency gate with polling and timeout |
