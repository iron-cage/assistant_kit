# Parameter: `base_image`

- **Status:** ✅ Configured — via `runbox.yml`; default: `rust:slim`
- **Current State:** `rust:slim`
- **Where It Flows:** `runbox.yml base_image:` → `--build-arg BASE_IMAGE` → `FROM $BASE_IMAGE AS chef` and `FROM $BASE_IMAGE AS test` in `runbox.dockerfile`

### Notes

`planner` and `cook` inherit from `chef` so they need no separate `FROM` — changing the chef base propagates to them automatically. Only the two explicit `FROM` lines (chef and test) must match; keeping them on the same value avoids ABI mismatches between the stages.

### Example

Pinning to a specific Rust release:
```yaml
base_image: rust:1.78-slim
```
`docker-run` passes `--build-arg BASE_IMAGE=rust:1.78-slim` → dockerfile bakes `FROM rust:1.78-slim AS chef` and `FROM rust:1.78-slim AS test`. To revert to floating latest, remove the key or leave it commented out — `docker-run` defaults to `rust:slim`.
