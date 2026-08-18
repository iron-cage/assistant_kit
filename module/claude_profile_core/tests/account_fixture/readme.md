# tests/account_fixture/

Shared fixture module for the `account_*_test.rs` integration binaries. Lives in a
subdirectory so cargo does not auto-discover it as a test binary of its own.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `mod.rs` | Credential-store file builders shared by account test binaries |
