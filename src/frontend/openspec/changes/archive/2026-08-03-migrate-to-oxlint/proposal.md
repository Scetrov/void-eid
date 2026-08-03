## Why

The frontend currently depends on ESLint and its TypeScript/React plugin ecosystem, while the repository has upgraded to TypeScript 7. Oxlint provides a faster, TypeScript-7-compatible linting path with a simpler native configuration and can replace the existing ESLint toolchain without changing the application's runtime behavior.

## What Changes

- Replace the frontend ESLint command and configuration with Oxlint.
- Add the Oxlint development dependency and configure equivalent TypeScript, React, React Hooks, and React Refresh checks where supported.
- Remove ESLint, `typescript-eslint`, and ESLint-specific plugins/configuration that are no longer needed.
- Preserve the `npm run lint` entry point so local development and CI continue to use the same command.
- **BREAKING**: Lint findings and rule coverage may differ from ESLint; unsupported or intentionally changed rules must be documented and validated during migration.

## Capabilities

### New Capabilities

- `frontend-linting`: Run repository frontend lint checks through Oxlint with documented configuration and stable package-script behavior.

### Modified Capabilities

<!-- No existing OpenSpec capabilities are present; this is a new toolchain capability. -->

## Impact

- Affected files include `package.json`, the package lockfile, the ESLint configuration, and a new Oxlint configuration.
- Developer and CI lint commands remain `npm run lint`, but their implementation and diagnostics change.
- ESLint packages and plugins are removed from frontend development dependencies.
- Application source code may require targeted lint-compatible fixes if Oxlint exposes existing issues or if equivalent rules have different semantics.
