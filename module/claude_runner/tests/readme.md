# tests/

| File | Responsibility |
|------|----------------|
| `commands_yaml_test.rs` | Verify YAML defines `.claude`, `.claude.help`, and rejects `.please`. |
| `cli_args_test.rs` | CLI flag parsing: core flags T01–T49, correct translation to builder calls. |
| `ultrathink_args_test.rs` | Ultrathink suffix injection and `--no-ultrathink` opt-out (T50–T58). |
| `effort_args_test.rs` | Effort flag defaults, overrides, suppression, and corner cases (T59–T70). |
| `dry_run_test.rs` | Dry-run output: env vars and command line structure. |
| `execution_mode_test.rs` | Execution modes: interactive/print paths, error handling, exit codes. |
| `verbosity_test.rs` | Verbosity flag: output gating levels 0–5, default behavior. |
| `lib_test.rs` | Library API: `register_commands()` callable. |
| `stale_ref_guard_test.rs` | Guard against stale `claude_runner_plugin` and `dream_agent` references. |
| `manual/readme.md` | Manual testing plan: live Claude Code invocation. |
