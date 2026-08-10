# Void eID Frontend

The frontend is a React 19 and Vite application. **Bun 1.3.14 is the canonical and only supported package manager.** The committed `bun.lock` is the source of truth for dependency resolution; do not create or commit `package-lock.json`, `yarn.lock`, or `pnpm-lock.yaml`.

## Prerequisite

Install [Bun 1.3.14](https://bun.com/docs/installation) and verify the version:

```bash
bun --version
# 1.3.14
```

CI, development containers, and frontend container images use the same pinned release.

## Install dependencies

From `src/frontend`:

```bash
bun install --frozen-lockfile
```

Use a frozen install for normal development and validation. When intentionally changing dependencies, use `bun add`, `bun remove`, or `bun update` and commit both `package.json` and `bun.lock`.

```bash
bun add <package>
bun add --dev <package>
bun remove <package>
bun update <package>
```

## Common commands

```bash
bun run dev       # Start the Vite development server
bun run lint      # Run Oxlint
bun run test      # Run Vitest
bun run build     # Build the production bundle
bun run test:e2e  # Build the stub API and run Playwright
```

Use `bun x <binary>` for a package binary that does not have a package script, for example `bun x playwright test`.

## Dependency updates

Dependabot is configured for the Bun ecosystem and must update both `package.json` and `bun.lock`. A dependency pull request is not ready to merge if `bun install --frozen-lockfile` fails.
