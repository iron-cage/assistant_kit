# Parameter: `image`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** per-module tag (e.g., `claude_tools_test`, `dream_test`)
- **Where It Flows:** `docker build -t $IMAGE $WORKSPACE` → `docker run $IMAGE`

### Notes

Unique per module. Prevents image tag collisions between workspace-level and crate-level builds.
