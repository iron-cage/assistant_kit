# Doc Entities

## Master Doc Entities Table

| Entity | Purpose | Master File | Instances |
|--------|---------|-------------|----------:|
| `api/` | Public API contract for execution control | [api/readme.md](api/readme.md) | 1 |
| `claude_params/` | Claude CLI parameter specifications (60 params) | [claude_params/readme.md](claude_params/readme.md) | 60 |
| `data_structure/` | Domain type documentation for command builder | [data_structure/readme.md](data_structure/readme.md) | 1 |
| `failure_mode/` | Documented failure modes and silent error conditions | [failure_mode/readme.md](failure_mode/readme.md) | 4 |
| `feature/` | Behavioral requirements for execution control | [feature/readme.md](feature/readme.md) | 6 |
| `invariant/` | Measurable constraints for execution behavior | [invariant/readme.md](invariant/readme.md) | 2 |
| `pattern/` | Reusable design patterns in the runner core | [pattern/readme.md](pattern/readme.md) | 1 |
| `tests/docs/error/` | Per-error condition test case specifications | [../../tests/docs/error/readme.md](../../tests/docs/error/readme.md) | 0 |
| `tests/docs/feature/` | Per-feature test case specifications | [../../tests/docs/feature/readme.md](../../tests/docs/feature/readme.md) | 0 |

## Master Doc Instances Table

| Entity | ID | Name | File |
|--------|----|------|------|
| api | 001 | Execution API | [api/001_execution_api.md](api/001_execution_api.md) |
| data_structure | 001 | Command Types | [data_structure/001_command_types.md](data_structure/001_command_types.md) |
| failure_mode | 001 | Rate Limit Exit 2 | [failure_mode/001_rate_limit_exit_2.md](failure_mode/001_rate_limit_exit_2.md) |
| failure_mode | 002 | Diagnostic on Stdout | [failure_mode/002_diagnostic_on_stdout.md](failure_mode/002_diagnostic_on_stdout.md) |
| failure_mode | 003 | CLAUDE_CODE Env Leak | [failure_mode/003_claudecode_env_leak.md](failure_mode/003_claudecode_env_leak.md) |
| failure_mode | 004 | Exit 1 Ambiguity | [failure_mode/004_exit_1_ambiguity.md](failure_mode/004_exit_1_ambiguity.md) |
| feature | 001 | Execution Control | [feature/001_execution_control.md](feature/001_execution_control.md) |
| feature | 002 | Dry Run | [feature/002_dry_run.md](feature/002_dry_run.md) |
| feature | 003 | Describe | [feature/003_describe.md](feature/003_describe.md) |
| feature | 004 | Run Isolated | [feature/004_run_isolated.md](feature/004_run_isolated.md) |
| feature | 005 | Stdin File | [feature/005_stdin_file.md](feature/005_stdin_file.md) |
| feature | 006 | Unset CLAUDE_CODE | [feature/006_unset_claudecode.md](feature/006_unset_claudecode.md) |
| invariant | 001 | Single Execution Point | [invariant/001_single_execution_point.md](invariant/001_single_execution_point.md) |
| invariant | 002 | NFR Conformance | [invariant/002_nfr_conformance.md](invariant/002_nfr_conformance.md) |
| pattern | 001 | Command Builder | [pattern/001_command_builder.md](pattern/001_command_builder.md) |

> `claude_params` instances (60 files) use numbered naming and are enumerated in their master file: [claude_params/readme.md](claude_params/readme.md).
