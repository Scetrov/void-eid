## Context

The backend currently imports `sui-sdk` only from `src/backend/src/wallet.rs` to validate Sui addresses and verify wallet personal-message signatures during `/api/wallets/link-nonce` and `/api/wallets/link-verify`. That narrow usage pulls in the full Sui SDK dependency graph, including `sui-types` and the compromised `mysten-metrics` crate. The frontend already produces standard Sui personal-message signatures through `useSignPersonalMessage`, so the backend does not need RPC clients, transaction builders, Move support, or metrics infrastructure.

Current behavior to preserve:

- wallet addresses are normalized to lowercase and must parse as 32-byte Sui hex addresses;
- signatures are base64-encoded Sui signature bytes;
- supported signature schemes are the same as `sui_types::crypto::Signature`: Ed25519, Secp256k1, and Secp256r1;
- verification uses Sui personal-message intent semantics over the stored nonce;
- existing API responses, nonce lifecycle, network validation, duplicate-wallet handling, relinking, and audit logging remain unchanged.

## Goals / Non-Goals

**Goals:**

- Remove the direct backend `sui-sdk` dependency.
- Ensure `mysten-metrics` is absent from the resolved backend dependency graph.
- Preserve existing Sui wallet-linking behavior for Ed25519, Secp256k1, and Secp256r1 personal-message signatures.
- Keep the verification implementation small, auditable, and isolated from route/database logic.
- Add regression tests that compare expected success and failure behavior around address parsing and signature verification.

**Non-Goals:**

- No frontend wallet-link flow changes.
- No endpoint, request, response, or database schema changes.
- No support expansion for zkLogin, multisig, passkey, transaction signing, or Sui RPC operations.
- No replacement of the entire Sui SDK surface.

## Decisions

### Replace `sui-sdk` with a local verifier module

Create a focused backend module, for example `src/backend/src/sui_verify.rs`, that exposes one route-level function such as `verify_sui_personal_message(address, signature_base64, message_bytes)`. `wallet.rs` should delegate verification to this module and keep existing nonce/database behavior unchanged.

Rationale: the application only needs Sui personal-message verification, not the SDK. Isolating the logic makes the security-sensitive replacement reviewable and avoids spreading protocol details through handlers.

Alternatives considered:

- Keep `sui-sdk` and wait for upstream remediation: rejected because it leaves the compromised crate in the dependency graph and retains a large unused surface.
- Depend on `sui-types` directly: rejected because `sui-types` itself pulls `mysten-metrics`.
- Disable wallet linking temporarily: rejected because it changes functionality.

### Implement only the currently supported signature schemes

The local verifier will parse the Sui signature format used by `sui_types::crypto::Signature`: `flag || signature || public_key`. It will support Ed25519, Secp256k1, and Secp256r1, matching the current enum used by the backend. Unknown flags or incorrect lengths must fail verification.

Rationale: preserving current functionality means matching the existing `Signature` behavior, not broadening to `GenericSignature` variants such as multisig, zkLogin, or passkeys.

### Preserve Sui address derivation and intent hashing

For each supported scheme, derive the signer address as Sui does: Blake2b-256 over `scheme_flag || public_key`, compared to the parsed 32-byte address. Build the same personal-message intent payload for the nonce and verify the signature over the Blake2b digest of the BCS-serialized intent message.

The replacement must use fixture tests generated from the current SDK-backed implementation or equivalent known-good vectors before `sui-sdk` is removed, so the new code is proven byte-compatible for the supported schemes.

### Prefer narrow crypto/hash dependencies

Use already-present direct dependencies where possible (`base64`, `bcs`, `serde`) and add only narrowly scoped crypto/hash crates needed for verification. If `fastcrypto` is retained directly, configure it with the minimum practical feature set and verify it does not retain `mysten-metrics` through the dependency graph.

## Risks / Trade-offs

- [Risk] Personal-message serialization or hashing diverges from Sui semantics → Mitigation: create SDK-backed fixtures before removal and assert the local verifier accepts the same valid signatures and rejects altered messages, addresses, and signatures.
- [Risk] Some real wallet uses an unsupported signature variant → Mitigation: preserve exactly the variants currently accepted by `sui_types::crypto::Signature`; document that this change does not add `GenericSignature` support.
- [Risk] A replacement crypto dependency still pulls vulnerable Mysten crates → Mitigation: run `cargo tree -i mysten-metrics --locked` and `cargo tree -i sui-sdk --locked` after lockfile regeneration.
- [Risk] Error text or status codes drift during refactor → Mitigation: keep handler-level error mapping in `wallet.rs` and make verifier errors opaque to clients.

## Migration Plan

1. Add SDK-backed fixture tests or vectors for current personal-message verification behavior.
2. Implement the local Sui verifier and route it through `wallet.rs`.
3. Remove `sui-sdk` from `Cargo.toml` and regenerate `Cargo.lock`.
4. Run backend tests and dependency-tree checks proving `sui-sdk` and `mysten-metrics` are absent.
5. Rollback strategy: revert the dependency and verifier changes if signature verification fails in testing; no data migration is involved.

## Open Questions

- Should the implementation keep `shared-crypto` if it remains free of `mysten-metrics`, or inline the small intent structure as well to reduce Mysten dependencies further? The preferred starting point is to remove `sui-sdk` first and verify the dependency tree.
