## Context

The frontend currently uses ESLint flat config with `@eslint/js`, `typescript-eslint`, browser globals, React Hooks, and React Refresh rules. The `lint` package script is consumed by developers and likely CI, while the application has already moved to TypeScript 7. The migration must change the lint implementation without changing runtime code or the public developer command.

## Goals / Non-Goals

**Goals:**

- Make Oxlint the sole frontend lint runner.
- Preserve `npm run lint` as the supported entry point and ensure it checks the intended source tree while excluding generated output.
- Establish explicit rule-equivalence decisions for TypeScript, React Hooks, React Refresh, and the existing route-specific exception.
- Remove obsolete ESLint configuration and dependencies after the Oxlint check is validated.
- Provide a repeatable validation path for local development and CI.

**Non-Goals:**

- Changing application behavior, formatting, TypeScript compiler settings, or build/test tooling.
- Introducing a formatter or replacing TypeScript type-checking with linting.
- Reproducing every ESLint rule when Oxlint has no supported equivalent; such gaps must instead be documented and assessed.

## Decisions

1. **Use Oxlint as the lint command behind the existing script.**
   - `npm run lint` remains stable for consumers, while its implementation invokes Oxlint with an explicit configuration and the repository's intended file scope.
   - Alternative considered: rename the script to `oxlint`; rejected because it needlessly breaks CI and contributor workflows.

2. **Keep lint configuration in a committed Oxlint config file.**
   - The config will ignore generated directories such as `dist`, enable supported JavaScript/TypeScript and React-oriented rules, and encode any intentional route-specific exception in a reviewable location.
   - Alternative considered: rely entirely on CLI defaults; rejected because defaults do not document repository policy and make future upgrades harder to review.

3. **Map rules by behavior, not by package name.**
   - Existing ESLint findings and configuration will be inventoried, then each rule will be mapped to an Oxlint equivalent, replaced with a deliberate alternative, or recorded as an accepted coverage gap.
   - Alternative considered: mechanically translate the ESLint config; rejected because plugin and rule semantics are not one-to-one.

4. **Remove ESLint packages only after parity validation.**
   - The migration will first install and run Oxlint, resolve or document findings, update scripts/configuration, and run lint plus build/tests before deleting ESLint-only packages and lockfile entries.
   - Alternative considered: remove ESLint first; rejected because it makes comparison and rollback more difficult.

5. **Pin the selected Oxlint release through the repository package manager.**
   - The implementation will select the latest applicable released version compatible with the project and lock it in the package manifest/lockfile according to repository dependency policy.
   - Alternative considered: use an unpinned executable download; rejected because it weakens reproducibility and supply-chain review.

## Risks / Trade-offs

- **[Rule coverage differs]** → Produce a rule mapping/coverage review and add targeted validation for important React and TypeScript checks.
- **[Oxlint reports new findings]** → Fix genuine source issues, or document narrowly scoped suppressions with rationale; do not broadly disable linting.
- **[Plugin support differs, especially React Hooks/Refresh]** → Verify supported Oxlint rules before implementation and record unsupported checks as explicit follow-up risk.
- **[Developer/CI behavior changes]** → Keep the `lint` script name stable and run the same command in validation and CI-oriented checks.
- **[Rollback needed]** → Keep the migration changes isolated so restoring the prior ESLint config and dependencies is a straightforward revert.

## Migration Plan

1. Inventory current ESLint rules, ignores, scripts, and representative lint output.
2. Add and configure the compatible Oxlint release; run it alongside ESLint for comparison.
3. Resolve findings and document rule mappings, intentional exceptions, and coverage gaps.
4. Switch `npm run lint` to Oxlint and validate lint, type-check/build, unit tests, and relevant end-to-end checks.
5. Remove ESLint configuration and unused dependencies, refresh the lockfile, and rerun validation.
6. Roll back by reverting the migration commit(s), restoring the ESLint script/configuration and dependency entries.

## Open Questions

- Which Oxlint release and React plugin support provide the closest maintained coverage for the current React Hooks and React Refresh rules at implementation time?
- Does CI invoke `npm run lint` indirectly through another workspace or root-level command that also needs updating?
- Are any ESLint rules currently relied upon outside the frontend package and therefore not safe to remove globally?
