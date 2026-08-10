## 1. Package Manager Configuration

- [x] 1.1 Declare Bun 1.3.14 in the frontend package manifest and remove `package-lock.json`
- [x] 1.2 Configure Dependabot to use the Bun ecosystem for the frontend
- [x] 1.3 Pin GitHub Actions to Bun 1.3.14 instead of `latest`

## 2. Development and Test Tooling

- [x] 2.1 Add the SHA-pinned Bun executable to the development container and use frozen Bun installation
- [x] 2.2 Replace active npm and npx invocations in frontend test tooling with Bun
- [x] 2.3 Prevent the frontend npm lockfile from being reintroduced

## 3. Documentation and Specifications

- [x] 3.1 Document Bun installation, version, dependency, and package-script commands for contributors
- [x] 3.2 Update active frontend linting requirements to use Bun commands

## 4. Validation

- [x] 4.1 Verify no active frontend workflow or documentation uses npm, npx, pnpm, or Yarn
- [x] 4.2 Run frozen dependency installation, lint, unit tests, and production build with Bun 1.3.14
- [x] 4.3 Validate OpenSpec artifacts and review the final repository diff
