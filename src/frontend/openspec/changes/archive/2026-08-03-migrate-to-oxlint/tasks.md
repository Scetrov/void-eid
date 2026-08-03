## 1. Inventory and select the Oxlint toolchain

- [x] 1.1 Inventory the current ESLint rules, ignores, scripts, package usage, and representative lint output.
- [x] 1.2 Verify the latest applicable released Oxlint version and its TypeScript, React Hooks, and React Refresh rule support.
- [x] 1.3 Create a rule-coverage map recording equivalent rules, deliberate replacements, unsupported checks, and required route-specific behavior.

## 2. Configure Oxlint

- [x] 2.1 Add the selected Oxlint release to frontend development dependencies and update the lockfile.
- [x] 2.2 Add the committed Oxlint configuration with the intended source scope and `dist` exclusion.
- [x] 2.3 Enable supported TypeScript and React lint rules and document accepted coverage gaps or narrowly scoped suppressions.
- [x] 2.4 Preserve or deliberately replace the existing React Refresh exception for `src/routes`.

## 3. Switch the repository lint workflow

- [x] 3.1 Update `npm run lint` to invoke Oxlint while preserving the existing command interface.
- [x] 3.2 Run Oxlint against the frontend and fix genuine findings without weakening unrelated checks.
- [x] 3.3 Remove the ESLint configuration and unused ESLint, TypeScript ESLint, and ESLint plugin dependencies.
- [x] 3.4 Confirm the final manifest and lockfile contain no dependency required only by the removed ESLint workflow.

## 4. Validate and document the migration

- [x] 4.1 Verify a clean dependency installation can run `npm run lint` without globally installed tooling.
- [x] 4.2 Run `npm run lint`, the TypeScript/build validation, unit tests, and relevant end-to-end checks.
- [x] 4.3 Verify generated output remains excluded and review the final rule-coverage map for completeness.
- [x] 4.4 Document any intentional behavior differences, unsupported checks, and follow-up risks in the migration change or repository documentation.
