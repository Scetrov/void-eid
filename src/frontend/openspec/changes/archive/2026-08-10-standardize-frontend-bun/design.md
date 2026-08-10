## Context

The frontend has both `bun.lock` and `package-lock.json`. CI and frontend container images use Bun with frozen-lockfile enforcement, but Dependabot and the development container use npm, while some test tooling invokes `npm`/`npx`. The repository already pins Bun 1.3.14 container images, and Bun 1.3.14 is the current applicable release.

## Goals / Non-Goals

**Goals:**
- Establish one package manager and one lockfile for frontend dependency resolution.
- Ensure Dependabot updates the lockfile consumed by CI.
- Use the same Bun release across package metadata, CI, and containers.
- Document reproducible Bun commands for contributors and automation.

**Non-Goals:**
- Change frontend dependencies except where lockfile normalization requires it.
- Change application behavior or build tooling.
- Rewrite historical archived OpenSpec artifacts.

## Decisions

- Use Bun 1.3.14 because it is already SHA-pinned in frontend container images and is the latest validated applicable release. Pin CI and `package.json` to the same version instead of using `latest`.
- Configure Dependabot with the native `bun` ecosystem so dependency pull requests update the text `bun.lock` consumed by frozen installs.
- Remove `package-lock.json` rather than maintaining two dependency-resolution graphs. Add a frontend-specific ignore rule to prevent accidental recommits.
- Add Bun to the development container by copying its executable from the existing SHA-pinned Bun image, avoiding an unverified remote installation script.
- Replace active npm/npx command invocations with `bun run` or `bun x`. Historical archived change records remain immutable.

## Risks / Trade-offs

- [Contributors only have npm installed] → Document the pinned Bun requirement and provide a clear installation reference.
- [Dependabot Bun behavior differs from npm grouping] → Preserve the existing schedule/group policy and validate the Dependabot configuration syntax.
- [Removing package-lock affects npm-only workflows] → Replace every active npm install/execution path and verify no active automation consumes the npm lockfile.
- [Bun version drift] → Pin 1.3.14 in package metadata, CI, and container images and update these together.

## Migration Plan

1. Update package metadata, automation, scripts, and documentation to Bun 1.3.14.
2. Remove `package-lock.json` and regenerate/verify `bun.lock` with the pinned Bun release.
3. Run frozen installation, lint, unit tests, build, and relevant configuration checks.
4. Sync the delta specs and archive this change before committing.

Rollback restores `package-lock.json`, npm commands, and the npm Dependabot ecosystem from the prior commit.

## Open Questions

None.
