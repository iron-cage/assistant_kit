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

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `filePath` | string | yes | Path to the ONBOARDING.md file |

### Since

v2.0+ (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
