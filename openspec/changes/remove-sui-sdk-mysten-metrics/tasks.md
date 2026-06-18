## 1. Verification Fixtures

- [x] 1.1 Add SDK-backed regression fixtures or equivalent known-good vectors for Sui personal-message verification before removing `sui-sdk`
- [x] 1.2 Cover successful Ed25519 verification for a nonce signed by the matching Sui address
- [x] 1.3 Cover wrong address, wrong message, malformed base64, malformed signature length, and unknown signature flag rejection
- [x] 1.4 Add Secp256k1 and Secp256r1 vectors or document if project test tooling cannot generate them locally

## 2. Local Sui Verifier

- [x] 2.1 Create a focused backend verifier module for Sui address parsing and personal-message signature verification
- [x] 2.2 Parse Sui hex addresses as exactly 32 bytes with existing lowercase normalization preserved at the handler boundary
- [x] 2.3 Parse Sui signature bytes as `flag || signature || public_key` for Ed25519, Secp256k1, and Secp256r1
- [x] 2.4 Reproduce Sui address derivation as Blake2b-256 over `scheme_flag || public_key`
- [x] 2.5 Reproduce Sui personal-message intent serialization and digest generation for nonce bytes
- [x] 2.6 Verify signatures with narrow crypto dependencies and return opaque verifier errors suitable for existing handler mapping

## 3. Route Integration

- [x] 3.1 Replace `sui-sdk` imports in `src/backend/src/wallet.rs` with calls to the local verifier module
- [x] 3.2 Preserve existing `/api/wallets/link-nonce` invalid-address behavior
- [x] 3.3 Preserve existing `/api/wallets/link-verify` error status codes and client-facing error messages
- [x] 3.4 Confirm nonce removal, TTL handling, network validation, duplicate-wallet handling, relinking, and audit logging are unchanged

## 4. Dependency Removal

- [x] 4.1 Remove `sui-sdk` from `src/backend/Cargo.toml`
- [x] 4.2 Remove `shared-crypto` too if the local verifier no longer needs it and doing so preserves behavior
- [x] 4.3 Add only narrowly scoped direct crypto/hash dependencies required by the verifier
- [x] 4.4 Regenerate `src/backend/Cargo.lock`
- [x] 4.5 Verify `cargo tree -i sui-sdk --locked` no longer resolves `sui-sdk`
- [x] 4.6 Verify `cargo tree -i mysten-metrics --locked` no longer resolves `mysten-metrics`

## 5. Validation

- [x] 5.1 Run backend unit tests for wallet linking and the new verifier module
- [x] 5.2 Run `cargo test --locked` from `src/backend`
- [x] 5.3 Run `cargo check --locked` from `src/backend`
- [ ] 5.4 Manually verify the frontend wallet-link flow against the backend if a browser wallet/test wallet environment is available
- [x] 5.5 Document any dependency alert that cannot be fully removed without functionality reduction (none remaining; no documentation required)
