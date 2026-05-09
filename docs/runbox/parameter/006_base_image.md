# Parameter: `base_image`

- **Status:** 🔒 Hardcoded — in `runbox.dockerfile`
- **Current State:** `rust:slim`
- **Where It Flows:** `FROM rust:slim` in all four build stages (chef, planner, cook, test)

### Notes

Version-unpinned. Identical string baked into both the chef and test stages; any change must be applied consistently to all four `FROM` lines.

### Example

```dockerfile
FROM rust:slim AS chef      # installs cargo-chef; inherited by planner and cook
FROM chef AS planner        # inherits rust:slim via chef — no separate FROM needed
FROM chef AS cook           # inherits rust:slim via chef
FROM rust:slim AS test      # final image; fresh rust:slim, receives cook artifacts
```
To pin to a specific version, change `rust:slim` → `rust:1.78-slim` in the `chef` and `test` FROM lines. Planner and cook inherit from `chef`, so they update automatically. A mismatch (chef on 1.77, test on 1.78) risks ABI-incompatible artifacts if the rust ABI changes between patch versions — keep both FROM lines on the same value.
