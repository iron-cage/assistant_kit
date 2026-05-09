# Parameter: `rustup_components`

- **Status:** 🔒 Hardcoded — in `runbox.dockerfile`
- **Current State:** `clippy`
- **Where It Flows:** `rustup component add clippy` in test stage

### Notes

Single component hardcoded. Projects needing `rustfmt` or `llvm-tools-preview` must add them manually or make this parameter configurable.
