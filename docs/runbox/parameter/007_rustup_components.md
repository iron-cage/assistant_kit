# Parameter: `rustup_components`

- **Status:** 🔒 Hardcoded — in `runbox.dockerfile`
- **Current State:** `clippy`
- **Where It Flows:** `rustup component add clippy` in test stage

### Notes

Single component hardcoded. Projects needing `rustfmt` or `llvm-tools-preview` must add them manually or make this parameter configurable.

### Example

```dockerfile
RUN rustup component add clippy
```
`w3 .test level::3` runs `cargo clippy --all-targets --all-features -- -D warnings` inside the container. Without `clippy` in the test stage, this fails with `error: unknown component: clippy`. To add `rustfmt` for format-check tests, append `RUN rustup component add rustfmt` to the test stage — there is no `runbox.yml` key; it is a direct dockerfile edit.
