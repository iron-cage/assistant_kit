# Pattern: Version Pinning

### Scope

- **Purpose**: Explain how the official Claude Code mechanisms — plus one explicitly-flagged non-official recovery bridge — combine into a coherent strategy for fixing the running version instead of tracking `latest`.
- **Responsibility**: Synthesize release-channel selection, update floors/ceilings, install-method restriction, and install-time version selection into one pinning narrative, cross-referencing rather than duplicating each mechanism's own spec.
- **In Scope**: `autoUpdatesChannel`, `minimumVersion`, `requiredMinimumVersion`, `requiredMaximumVersion`, `installMethod`, `DISABLE_AUTOUPDATER`/`DISABLE_UPDATES`, install-time version selection (installer script, package managers, npm), manifest/code-signing verification as an integrity complement, and `preferredVersionSpec`/`preferredVersionResolved` as a non-official recovery bridge.
- **Out of Scope**: This repo's own `claude_version` lock implementation (→ `../../../../module/claude_version/docs/pattern/001_version_lock.md`); mechanics of any single parameter (→ the linked `param/`/`subcommand/`/`settings/` instances).

### Problem

Claude Code auto-updates by default, which suits most users but breaks reproducibility for anyone who needs a known, fixed version — a CI pipeline, a staged org rollout, or a developer avoiding a regression in the newest build. There is no single "pin" switch: control is spread across independent layers — per-user toggles, org-wide managed policy, install-time version choice, and package-manager-specific behavior — and no single layer is sufficient alone. Disabling the background updater still leaves `claude update` free to run manually; a hard version ceiling with no floor still allows arbitrary staleness; a policy pushed to managed settings can't reach a user who installs through a method the policy didn't restrict.

### Solution

Compose the layers from loosest to strictest:

1. **Channel selection** — `autoUpdatesChannel` (`latest` vs `stable`, see [../param/121_auto_updates_channel.md](../param/121_auto_updates_channel.md)) picks how aggressively background updates and `claude update` track new releases; the Homebrew cask choice (`claude-code` vs `claude-code@latest`) is the equivalent decision at install-method level, described in the same file.
2. **Soft update floor** — `minimumVersion` ([../param/122_minimum_version.md](../param/122_minimum_version.md)) is the first rung with real teeth: it puts a numeric floor under auto-update and `claude update`. Like channel selection, it can be set directly by a user or pushed org-wide via managed settings for a non-overridable floor — but even managed, it only constrains what gets installed next; an already-installed older binary keeps running regardless of the floor, which is what actually separates it from the startup-blocking `requiredMinimumVersion` below.
3. **Hard organizational bounds** — `requiredMinimumVersion` and `requiredMaximumVersion` ([../param/123_required_minimum_version.md](../param/123_required_minimum_version.md), [../param/124_required_maximum_version.md](../param/124_required_maximum_version.md)), managed-settings only (both added v2.1.163): Claude Code refuses to start outside this range. Both are designed as a bound the organization controls, not a way to strand a machine — see the linked instances for exactly how an invalid policy value is neutralized and which commands stay available to recover.
4. **Update suppression** — `env.DISABLE_AUTOUPDATER` (background checks only) vs `env.DISABLE_UPDATES` (background and manual) ([../param/099_disable_autoupdater.md](../param/099_disable_autoupdater.md), [../param/119_disable_updates.md](../param/119_disable_updates.md)) stop the updater from acting at all, independent of channel or floor/ceiling. `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` ([../param/126_disable_nonessential_traffic.md](../param/126_disable_nonessential_traffic.md)) bundles `DISABLE_AUTOUPDATER` with three unrelated opt-outs for the "eliminate all non-essential traffic" case.
5. **Install-method restriction** — `installMethod` in managed `settings.json` restricts which install methods (native, Homebrew, npm, etc.) an organization permits at all. No dedicated parameter instance exists yet for this key; it is described here because it closes the specific loophole where a version floor/ceiling is set but a user can route around it by reinstalling through a method the policy didn't anticipate.
6. **Install-time version selection** — the mechanism that actually places a specific binary on disk:
   - `claude install [target] [--force]` ([../subcommand/005_install.md](../subcommand/005_install.md)) selects `stable`, `latest`, or a specific version once Claude Code is already present on the system.
   - The bootstrap installer script (invoked as `curl <install-url> | bash -s -- <version>`) is the equivalent operation for a machine that does not have Claude Code yet — it is an official Anthropic install path, but no doc instance in this collection gives it its own dedicated coverage yet (this repo's own [`module/claude_version/docs/feature/001_version_management.md`](../../../../module/claude_version/docs/feature/001_version_management.md) narrates the same invocation, but only as an implementation detail of `claude_version`'s own `.version.install` command); its post-install behavior performs the same version selection `claude install` performs.
   - Package-manager-owned selection: Homebrew/WinGet/apt/dnf/apk each make their own version/channel decision at their own repo, cask, or package-config layer, outside anything Claude Code's own settings can override. `CLAUDE_CODE_PACKAGE_MANAGER_AUTO_UPDATE` ([../param/125_package_manager_auto_update.md](../param/125_package_manager_auto_update.md), added v2.1.129) is the one exception: it can opt Homebrew or WinGet back into auto-upgrading, though it doesn't touch which version or channel they choose — apt/dnf/apk have no equivalent hook at all.
   - npm global installs pin with ordinary npm syntax: `npm install -g @anthropic-ai/claude-code@<version|tag>` — like the bootstrap script above, this is an official install path with no dedicated doc instance in this collection yet.
7. **Recovery bridge (documented as non-official)** — `preferredVersionSpec` and `preferredVersionResolved` ([../param/050_preferred_version_spec.md](../param/050_preferred_version_spec.md), [../param/049_preferred_version_resolved.md](../param/049_preferred_version_resolved.md), [../settings/003_version_lock.md](../settings/003_version_lock.md)) are documented, per their own spec, as not read by the `claude` binary itself at runtime — existing purely as `claude_version`'s own bookkeeping, written into the same `settings.json` the official binary owns. (Note: [../subcommand/009_update.md](../subcommand/009_update.md) separately states that `claude update` "respects the `preferredVersionSpec` setting if configured" — an apparent pre-existing tension between these two doc instances that this file inherits rather than resolves.) Their relevance here is downstream: this repo's tooling reads them to detect drift and reassert a pin via the mechanisms above after everything else is bypassed; see [../../../../module/claude_version/docs/pattern/001_version_lock.md](../../../../module/claude_version/docs/pattern/001_version_lock.md) for how they're actually used.
8. **Integrity complement** — none of the above confirms the binary occupying a pinned version slot is genuine. Manifest verification (`manifest.json`) plus platform code-signing (GPG on Linux, `codesign` on macOS, Authenticode on Windows) close that gap. No doc instance in this collection covers this integrity layer yet — it's included here because a pin is only meaningful if the bytes behind the pinned version number can be trusted, not because a source document verifies these specific mechanism names. This is not a pinning mechanism itself — it answers "is this really the build it claims to be," not "which version runs."

### Applicability

Applies whenever Claude Code must stay on a known version: CI reproducibility, staged organizational rollouts, or avoiding a specific regression. Pick the loosest sufficient layer — channel selection alone is enough for "don't take bleeding-edge by default"; add a floor/ceiling when the constraint must survive a user's own settings changes; add `installMethod` and `DISABLE_UPDATES` when even a determined user must not be able to escape the pin. Does not apply when tracking `latest` is desired — that is the unconfigured default. `requiredMinimumVersion`, `requiredMaximumVersion`, and `installMethod` are managed-settings-only, with no self-service equivalent; an individual user without that deployment path falls back to channel selection, the soft floor, and per-user env vars in their own settings — though an organization can also enforce channel selection and the soft floor through managed settings, short of the startup-blocking behavior that sets `requiredMinimumVersion`/`requiredMaximumVersion` apart. This pattern covers the official, upstream mechanisms, plus one flagged exception (item 7's recovery bridge — see item 7 for a noted pre-existing tension in its sourcing); this repo's own `claude_version` tool adds a further enforcement layer on top (filesystem lock, cached-binary purge, alias resolution) — see [../../../../module/claude_version/docs/pattern/001_version_lock.md](../../../../module/claude_version/docs/pattern/001_version_lock.md).

### Consequences

**Benefits:**
- Layers compose into defense in depth; each closes a bypass vector the others leave open.
- Org-level layers (`requiredMinimumVersion`/`requiredMaximumVersion`/`installMethod`) cannot be overridden by an individual user's own settings — and `minimumVersion`/`autoUpdatesChannel` can reach that same non-overridable status via managed settings, though only the former three block startup outright rather than just constraining updates.
- Recovery paths (`claude update`, `claude install`, `claude doctor`) stay reachable even while floors/ceilings are enforced, so pinning cannot itself strand a machine.

**Costs:**
- No single layer is sufficient; a correct pinning strategy requires composing several settings, and it is easy to leave a gap (e.g., a ceiling with no floor still permits arbitrary staleness).
- The strongest layers require managed-settings/MDM deployment and are unavailable to individual users.
- Fail-open behavior on malformed managed values avoids bricking a fleet but means a broken policy push is silently unenforced rather than loudly rejected.
- Package-manager-owned *version and channel selection* (Homebrew/WinGet/apt/dnf/apk) sits outside Claude Code's configuration surface — pinning there depends on OS-level package-manager mechanisms. `CLAUDE_CODE_PACKAGE_MANAGER_AUTO_UPDATE` is a narrow exception (Homebrew/WinGet auto-upgrade toggle only); it does not extend to apt/dnf/apk, and none of the five expose version/channel choice to a Claude Code setting.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master pattern table |
| doc | [../param/099_disable_autoupdater.md](../param/099_disable_autoupdater.md) | Background-only update suppression |
| doc | [../param/119_disable_updates.md](../param/119_disable_updates.md) | Full update suppression |
| doc | [../param/121_auto_updates_channel.md](../param/121_auto_updates_channel.md) | Release channel selection |
| doc | [../param/122_minimum_version.md](../param/122_minimum_version.md) | Soft update floor |
| doc | [../param/123_required_minimum_version.md](../param/123_required_minimum_version.md) | Hard startup floor (managed only) |
| doc | [../param/124_required_maximum_version.md](../param/124_required_maximum_version.md) | Hard startup ceiling (managed only) |
| doc | [../param/125_package_manager_auto_update.md](../param/125_package_manager_auto_update.md) | Homebrew/WinGet auto-upgrade opt-in |
| doc | [../param/126_disable_nonessential_traffic.md](../param/126_disable_nonessential_traffic.md) | Bundled non-essential traffic opt-out |
| doc | [../param/049_preferred_version_resolved.md](../param/049_preferred_version_resolved.md) | Resolved version recovery signal |
| doc | [../param/050_preferred_version_spec.md](../param/050_preferred_version_spec.md) | Preferred version spec recovery signal |
| doc | [../subcommand/005_install.md](../subcommand/005_install.md) | Install-time version selection |
| doc | [../subcommand/009_update.md](../subcommand/009_update.md) | Manual update/upgrade |
| doc | [../settings/003_version_lock.md](../settings/003_version_lock.md) | Preferred version storage and install sequence |
| doc | [../../../../module/claude_version/docs/pattern/001_version_lock.md](../../../../module/claude_version/docs/pattern/001_version_lock.md) | This repo's own enforcement layer built atop these mechanisms |
| doc | [../../../../module/claude_version/docs/feature/001_version_management.md](../../../../module/claude_version/docs/feature/001_version_management.md) | This repo's own use of the bootstrap installer script and the preferred-version recovery bridge |
