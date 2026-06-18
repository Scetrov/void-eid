## Why

The backend currently depends on `sui-sdk` only for Sui wallet address parsing and personal-message signature verification, but that dependency pulls in a large transitive Mysten/Sui graph including the compromised `mysten-metrics` crate. Removing the unnecessary SDK dependency reduces supply-chain exposure without changing wallet-linking functionality.

## What Changes

- Remove the backend's direct `sui-sdk` dependency.
- Remove the transitive `mysten-metrics` dependency from the resolved backend dependency graph.
- Replace the narrow `sui-sdk` usage in wallet linking with a local verifier that preserves current Sui personal-message verification behavior for Ed25519, Secp256k1, and Secp256r1 signatures.
- Preserve existing wallet-linking API behavior, request/response shapes, nonce handling, network handling, and database writes.
- Add verification coverage proving valid signatures still link wallets and invalid signatures are still rejected.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `security-remediation`: dependency vulnerability remediation must include removing vulnerable transitive crates when the application only needs a narrow subset of upstream functionality and can preserve behavior safely.

## Impact

- Affected backend files: `src/backend/Cargo.toml`, `src/backend/Cargo.lock`, `src/backend/src/wallet.rs`, and a new focused Sui signature verification module.
- Affected functionality: Sui wallet linking and signature verification only.
- Dependency impact: removes `sui-sdk` and the resolved `mysten-metrics` crate; may add narrowly scoped crypto/hash crates if not already available through remaining dependencies.
- API impact: none expected; existing frontend wallet-link flow and backend endpoints remain unchanged.
