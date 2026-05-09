# Plugin: Offline runner

- **Status:** 🔒 Hardcoded — baked into image `CMD` via `runbox.dockerfile` `ARG`
- **Controls:** What command the image's default `CMD` executes when run without `test_script`
- **Mechanism:** `cargo nextest run $CMD_SCOPE --filter-expr $CMD_FILTER` baked at image build time via dockerfile `ARG`

### Notes

Always nextest. Baked at build time — changing the offline runner requires rebuilding the image. The filter expression is also baked in at this point via `CMD_FILTER`.
