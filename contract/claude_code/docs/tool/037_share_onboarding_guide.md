# Tool: ShareOnboardingGuide

Upload ONBOARDING.md and return a shareable link.

### Category

Interaction

### Permission Required

Yes

### Description

Uploads an `ONBOARDING.md` file and returns a share link that teammates can open
in Claude Code. Called from the `/team-onboarding` command after the guide is
written. The link allows new team members to bootstrap their environment.

### Availability

Requires Pro, Max, Team, or Enterprise plan.

### Parameters

❌ **`filePath` refuted** — the doc previously claimed a required `filePath` string parameter. Confirmed wrong: v2.1.220's real schema is a `strictObject` (unknown keys rejected) with no path parameter at all — the tool always reads `ONBOARDING.md` implicitly from the current directory:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `mode` | enum | ❌ | `"check"` (default) — if a local `ONBOARDING.md` is present, uploads it to the most-recently-updated org guide (creates one if none exist); otherwise reports the existing link without uploading. `"update"` — upload to a specific guide identified by `short_code`. `"create"` — always make a new link. `"delete"` — remove a guide. |
| `short_code` | string | ❌ | Short code of a specific guide to target (returned by a previous call). Honored by `check`, `update`, and `delete` — skips the org-wide lookup and targets this guide directly. Pattern: `^[A-Za-z0-9_-]{1,64}$`. |

**Verify:** `grep -ac -F "Short code of a specific guide to target" ~/.local/share/claude/versions/2.1.220` → 1; the schema's own `v.strictObject({mode, short_code})` construction is visible verbatim adjacent to that string.

### Since

v2.0+ (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [028_artifact.md](028_artifact.md) | Artifact — publish HTML/Markdown files as shareable artifacts (related publishing tool) |
