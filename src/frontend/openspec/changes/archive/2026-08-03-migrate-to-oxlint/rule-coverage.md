# Oxlint migration rule coverage

## Inventory baseline

- Runner: `npm run lint` → `eslint .`.
- Source scope: `**/*.{ts,tsx}`; browser globals; ECMAScript 2020.
- Ignore: `dist`.
- Policy sources: `@eslint/js` recommended, `typescript-eslint` recommended, `eslint-plugin-react-hooks` recommended, and `eslint-plugin-react-refresh`'s `only-export-components` warning.
- Route behavior: `react-refresh/only-export-components` is disabled for `src/routes/**/*.{ts,tsx}`. Outside routes, the named export `Route` is allowed.
- Representative result: ESLint exits 2 before linting because `typescript-eslint` 8.65.0 rejects TypeScript 7.0.2.

## Selected toolchain

- Oxlint: exactly `1.77.0` (latest released npm version verified during implementation).
- Registry integrity: `sha512-qnGh8XJHaQ0dprrDXNQZgS0FgjI6v+V3+X8DwmaV++5Aamy6jGKfDdQ1TUvhUxtmKFAbEf4/WeO5QZX+5WSngg==`; the package lock records the resolved package integrity.
- Oxlint parses TypeScript natively and provides built-in `typescript` and `react` plugins. The React plugin includes React Hooks and React Refresh rules.
- A pinned transient run (`npx --yes oxlint@1.77.0`) validated the configuration before the incompatible ESLint dependency graph was removed.

## Coverage map

### ESLint recommended

The following rules have same-name Oxlint implementations and remain covered by Oxlint's default correctness policy:

`constructor-super`, `for-direction`, `getter-return`, `no-async-promise-executor`, `no-case-declarations`, `no-class-assign`, `no-compare-neg-zero`, `no-cond-assign`, `no-const-assign`, `no-constant-binary-expression`, `no-constant-condition`, `no-control-regex`, `no-debugger`, `no-delete-var`, `no-dupe-class-members`, `no-dupe-else-if`, `no-dupe-keys`, `no-duplicate-case`, `no-empty`, `no-empty-character-class`, `no-empty-pattern`, `no-empty-static-block`, `no-ex-assign`, `no-extra-boolean-cast`, `no-fallthrough`, `no-func-assign`, `no-global-assign`, `no-import-assign`, `no-invalid-regexp`, `no-irregular-whitespace`, `no-loss-of-precision`, `no-misleading-character-class`, `no-new-native-nonconstructor`, `no-nonoctal-decimal-escape`, `no-obj-calls`, `no-prototype-builtins`, `no-redeclare`, `no-regex-spaces`, `no-self-assign`, `no-setter-return`, `no-shadow-restricted-names`, `no-sparse-arrays`, `no-this-before-super`, `no-unassigned-vars`, `no-undef`, `no-unexpected-multiline`, `no-unreachable`, `no-unsafe-finally`, `no-unsafe-negation`, `no-unsafe-optional-chaining`, `no-unused-labels`, `no-unused-private-class-members`, `no-unused-vars`, `no-useless-assignment`, `no-useless-backreference`, `no-useless-catch`, `no-useless-escape`, `no-with`, `preserve-caught-error`, `require-yield`, `use-isnan`, and `valid-typeof`.

Deliberate replacements:

- `no-dupe-args`: already disabled for TypeScript by the prior `typescript-eslint` compatibility config; duplicate parameters remain rejected by TypeScript/Oxlint parsing.
- `no-octal`: replaced by TypeScript/Oxlint parser diagnostics for the TypeScript-only source scope.
- The prior TypeScript compatibility rules (`no-var`, `prefer-const`, `prefer-rest-params`, and `prefer-spread`) have native Oxlint equivalents enabled by its correctness policy.

### TypeScript recommended

Same-behavior built-in TypeScript equivalents are available for:

`@typescript-eslint/ban-ts-comment`, `@typescript-eslint/no-duplicate-enum-values`, `@typescript-eslint/no-empty-object-type`, `@typescript-eslint/no-explicit-any`, `@typescript-eslint/no-extra-non-null-assertion`, `@typescript-eslint/no-misused-new`, `@typescript-eslint/no-namespace`, `@typescript-eslint/no-non-null-asserted-optional-chain`, `@typescript-eslint/no-require-imports`, `@typescript-eslint/no-this-alias`, `@typescript-eslint/no-unnecessary-type-constraint`, `@typescript-eslint/no-unsafe-declaration-merging`, `@typescript-eslint/no-unsafe-function-type`, `@typescript-eslint/no-wrapper-object-types`, `@typescript-eslint/prefer-as-const`, `@typescript-eslint/prefer-namespace-keyword`, and `@typescript-eslint/triple-slash-reference`.

Deliberate canonical replacements:

- `@typescript-eslint/no-array-constructor` → Oxlint `no-array-constructor`.
- `@typescript-eslint/no-unused-expressions` → Oxlint `no-unused-expressions`, which handles TypeScript syntax.
- `@typescript-eslint/no-unused-vars` → Oxlint `no-unused-vars`, which handles TypeScript declarations and imports.

### React Hooks and React Refresh

- `react-hooks/rules-of-hooks` → `react/rules-of-hooks` (error).
- `react-hooks/exhaustive-deps` → `react/exhaustive-deps` (warning).
- The React Hooks compiler-oriented rules `static-components`, `use-memo`, `preserve-manual-memoization`, `incompatible-library`, `immutability`, `globals`, `refs`, `set-state-in-effect`, `error-boundaries`, `purity`, `set-state-in-render`, `unsupported-syntax`, `config`, and `gating` do not have separate Oxlint rule IDs. They are deliberately replaced by Oxlint's native `react/react-compiler` analysis (error). This is broader compiler analysis rather than one-to-one diagnostic identity.
- `react-refresh/only-export-components` → `react/only-export-components` (warning), retaining `allowExportNames: ["Route"]`.
- The `src/routes` override disables `react/only-export-components`, preserving the existing route-specific behavior.

## Intentional behavior differences and risks

- Oxlint's native default correctness set checks additional correctness rules beyond the old ESLint recommended sets. New genuine findings should be fixed rather than disabling the category broadly.
- React compiler diagnostics may differ in wording and location from the individual `eslint-plugin-react-hooks` compiler rules. The current repository passes the combined Oxlint compiler analysis, but upgrades should review compiler-related diagnostic changes.
- Linting remains syntax-aware rather than type-aware. Type-aware linting would require the separate `oxlint-tsgolint` package and is not part of the previous policy or this migration.
- The lint script scans the repository while configuration ignores JavaScript-family files, preserving the prior TypeScript/TSX-only scope. `dist/**` remains explicitly ignored.

## Validation

- `npm ci` followed by `npm run lint` succeeds without global tooling.
- `npm run build` succeeds, and the focused Vitest configuration runs the two unit test files (28 tests) without collecting Playwright specifications.
- An invalid temporary TypeScript probe under `dist` is ignored by Oxlint, confirming the generated-output exclusion.
- E2E validation succeeds in the pinned Playwright Dev Container: `podman run --rm --ipc=host --volume "$PWD/../..:/work:Z" --workdir /work/src/frontend localhost/void-eid-playwright-dev:local sh -lc 'npm ci && npm run test:e2e'` completed 57 Playwright tests successfully. The container supplies the 1.62.0 browser binaries, Rust, a C compiler, and OpenSSL headers; it avoids relying on the mismatched NixOS Playwright 1.61.1 package.
