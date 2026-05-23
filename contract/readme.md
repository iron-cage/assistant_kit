# Contract Tests

Behavioral contract test suites for external dependencies.

Each crate validates that an external binary or service upholds the behavioral
contract this workspace depends on. If the external system changes behavior,
the corresponding contract test goes RED.

| Directory | Responsibility |
|-----------|----------------|
| `claude_contract/` | Behavioral contract tests for the `claude` binary |
