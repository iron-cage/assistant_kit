# Parameter: `base_image`

- **Status:** 🔒 Hardcoded — in `runbox.dockerfile`
- **Current State:** `rust:slim`
- **Where It Flows:** `FROM rust:slim` in all four build stages (chef, planner, cook, test)

### Notes

Version-unpinned. Identical string baked into both the chef and test stages; any change must be applied consistently to all four `FROM` lines.
