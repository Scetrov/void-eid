use base64::{engine::general_purpose::STANDARD, Engine as _};
use blake2::{digest::consts::U32, Blake2b, Digest};
use serde::Serialize;
use sha2::{Digest as Sha2Digest, Sha256};
use signature::hazmat::PrehashVerifier;

/// Errors returned by the local Sui personal-message verifier.
/// Kept intentionally opaque so `wallet.rs` can map them to existing client-facing text.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidAddress,
    InvalidSignature,
    VerificationFailed,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidAddress => write!(f, "invalid Sui address"),
            Error::InvalidSignature => write!(f, "invalid Sui signature"),
            Error::VerificationFailed => write!(f, "signature verification failed"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Serialize)]
struct Intent {
    // Sui's Intent is serialized as scope, version, app_id.
    scope: u8,
    version: u8,
    app_id: u8,
}

#[derive(Serialize)]
struct IntentMessage<T: Serialize> {
    intent: Intent,
    value: T,
}

#[derive(Serialize)]
struct PersonalMessage {
    message: Vec<u8>,
}

/// Verify that `signature_base64` is a valid Sui personal-message signature over `message`
/// produced by the Sui address `address_hex`.
///
/// Supported signature schemes mirror the behavior previously obtained from
/// `sui_types::crypto::Signature`: Ed25519 (flag 0x00), Secp256k1 (flag 0x01), and
/// Secp256r1 (flag 0x02).
pub fn verify_sui_personal_message(
    address_hex: &str,
    signature_base64: &str,
    message: &[u8],
) -> Result<(), Error> {
    let expected_address = parse_address(address_hex)?;
    let signature = STANDARD
        .decode(signature_base64)
        .map_err(|_| Error::InvalidSignature)?;

    if signature.is_empty() {
        return Err(Error::InvalidSignature);
    }

    let digest = personal_message_digest(message);

    match signature[0] {
        0x00 => verify_ed25519(&signature, &digest, &expected_address),
        0x01 => verify_secp256k1(&signature, &digest, &expected_address),
        0x02 => verify_secp256r1(&signature, &digest, &expected_address),
        _ => Err(Error::InvalidSignature),
    }
}

/// Parse a Sui address from an optional `0x`-prefixed hex string.
/// The address must be exactly 32 bytes (64 hex chars) long.
pub fn parse_address(address: &str) -> Result<[u8; 32], Error> {
    let hex = address.strip_prefix("0x").unwrap_or(address);
    if hex.len() != 64 {
        return Err(Error::InvalidAddress);
    }
    let mut out = [0u8; 32];
    hex::decode_to_slice(hex, &mut out).map_err(|_| Error::InvalidAddress)?;
    Ok(out)
}

fn blake2b_256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Blake2b::<U32>::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

fn derive_address(scheme_flag: u8, public_key: &[u8]) -> [u8; 32] {
    let mut input = Vec::with_capacity(1 + public_key.len());
    input.push(scheme_flag);
    input.extend_from_slice(public_key);
    blake2b_256(&input)
}

fn personal_message_digest(message: &[u8]) -> [u8; 32] {
    let intent_msg = IntentMessage {
        intent: Intent {
            // Sui's Intent is serialized as scope, version, app_id.
            scope: 3,
            version: 0,
            app_id: 0,
        },
        value: PersonalMessage {
            message: message.to_vec(),
        },
    };
    let bytes = bcs::to_bytes(&intent_msg).expect("BCS serialization of intent message failed");
    blake2b_256(&bytes)
}

fn verify_ed25519(
    signature: &[u8],
    digest: &[u8; 32],
    expected_address: &[u8; 32],
) -> Result<(), Error> {
    if signature.len() != 1 + 64 + 32 {
        return Err(Error::InvalidSignature);
    }

    let sig = ed25519_dalek::Signature::from_slice(&signature[1..65])
        .map_err(|_| Error::InvalidSignature)?;
    let public_key_bytes: &[u8; 32] = signature[65..97]
        .try_into()
        .map_err(|_| Error::InvalidSignature)?;
    let public_key = ed25519_dalek::VerifyingKey::from_bytes(public_key_bytes)
        .map_err(|_| Error::InvalidSignature)?;

    public_key
        .verify_strict(digest, &sig)
        .map_err(|_| Error::VerificationFailed)?;

    let derived = derive_address(0x00, &signature[65..97]);
    if derived != *expected_address {
        return Err(Error::VerificationFailed);
    }

    Ok(())
}

fn verify_secp256k1(
    signature: &[u8],
    digest: &[u8; 32],
    expected_address: &[u8; 32],
) -> Result<(), Error> {
    if signature.len() != 1 + 64 + 33 {
        return Err(Error::InvalidSignature);
    }

    let ecdsa_sig = k256::ecdsa::Signature::from_slice(&signature[1..65])
        .map_err(|_| Error::InvalidSignature)?;
    let public_key = signature[65..98].to_vec();

    // Sui's Secp256k1 verifier passes the 32-byte Blake2b digest as the message
    // and then hashes it with SHA-256 before ECDSA verification.
    let ecdsa_prehash: [u8; 32] = Sha256::digest(digest).into();

    let verifying_key = k256::ecdsa::VerifyingKey::from_sec1_bytes(&public_key)
        .map_err(|_| Error::InvalidSignature)?;

    verifying_key
        .verify_prehash(&ecdsa_prehash, &ecdsa_sig)
        .map_err(|_| Error::VerificationFailed)?;

    let derived = derive_address(0x01, &public_key);
    if derived != *expected_address {
        return Err(Error::VerificationFailed);
    }

    Ok(())
}

fn verify_secp256r1(
    signature: &[u8],
    digest: &[u8; 32],
    expected_address: &[u8; 32],
) -> Result<(), Error> {
    if signature.len() != 1 + 64 + 33 {
        return Err(Error::InvalidSignature);
    }

    let ecdsa_sig = p256::ecdsa::Signature::from_slice(&signature[1..65])
        .map_err(|_| Error::InvalidSignature)?;
    let public_key = signature[65..98].to_vec();

    let ecdsa_prehash: [u8; 32] = Sha256::digest(digest).into();

    let verifying_key = p256::ecdsa::VerifyingKey::from_sec1_bytes(&public_key)
        .map_err(|_| Error::InvalidSignature)?;

    verifying_key
        .verify_prehash(&ecdsa_prehash, &ecdsa_sig)
        .map_err(|_| Error::VerificationFailed)?;

    let derived = derive_address(0x02, &public_key);
    if derived != *expected_address {
        return Err(Error::VerificationFailed);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        #[allow(dead_code)]
        name: &'static str,
        address: &'static str,
        signature: &'static str,
        nonce: &'static str,
    }

    // These fixtures were produced by the original SDK-backed implementation
    // (SuiKeyPair + Signature::new_secure over IntentMessage<PersonalMessage>)
    // before `sui-sdk` is removed. They serve as the known-good regression set.
    const FIXTURES: &[Fixture] = &[
        Fixture {
            name: "ed25519",
            address: "0x65b796666d2d98a9e2b6ec1e2750488a9b31e58e113f7d91b18aa9f197254d29",
            signature: "AHZxuVGL2l6SGJkyXrPnIT2cL7X8lxfBbitPaiF9qhxWpUkg5bVIdn7sVazeTgig+HVAF55wLEAt/WLbXDJKkQDmUlfr3GAOmWKBVjMKclVHpINYkijwijL82azNoFKkkg==",
            nonce: "test-nonce-1234",
        },
        Fixture {
            name: "secp256k1",
            address: "0x8a85de6c387a37933b3a6acfa2ac01c5c9f115e4e2f18c7fb14ef4cafae85ed4",
            signature: "AXEuKWnO70Ob6aMUyVv15LSGgHaENHYVcjeHrlgHhyqAFSdu8Y3QkSZSzGyI0J2sAfWfz3TLsZqm8U4Vi6aZSLkDX5sfgle5zuxoroYpGKdbTWkuayNX2KDSP0NvSfOx78k=",
            nonce: "test-nonce-1234",
        },
        Fixture {
            name: "secp256r1",
            address: "0x09e0cdfc9dbc0d2d636f3ed4d7121926863a760250771dea36c782f6055c17aa",
            signature: "AvHPoUHwrp6GF2qkmlvQQ16EciQBIwxCQ403Y6LqmXoQNhtxJZOp43vl1cCBhuBbMaKkJnAn9WXDpa60Is+mDmwDgv9of7DlNYAzV1vnxRv+uZEEhMIZkMQv9GPylGO7IpI=",
            nonce: "test-nonce-1234",
        },
    ];

    #[test]
    fn valid_ed25519_fixture_verifies() {
        let v = &FIXTURES[0];
        assert!(verify_sui_personal_message(v.address, v.signature, v.nonce.as_bytes()).is_ok());
    }

    #[test]
    fn valid_secp256k1_fixture_verifies() {
        let v = &FIXTURES[1];
        assert!(verify_sui_personal_message(v.address, v.signature, v.nonce.as_bytes()).is_ok());
    }

    #[test]
    fn valid_secp256r1_fixture_verifies() {
        let v = &FIXTURES[2];
        assert!(verify_sui_personal_message(v.address, v.signature, v.nonce.as_bytes()).is_ok());
    }

    #[test]
    fn wrong_address_rejected() {
        let v = &FIXTURES[0];
        let wrong_address = "0x0000000000000000000000000000000000000000000000000000000000000001";
        assert_eq!(
            verify_sui_personal_message(wrong_address, v.signature, v.nonce.as_bytes()),
            Err(Error::VerificationFailed)
        );
    }

    #[test]
    fn wrong_message_rejected() {
        let v = &FIXTURES[0];
        assert_eq!(
            verify_sui_personal_message(v.address, v.signature, b"wrong-nonce"),
            Err(Error::VerificationFailed)
        );
    }

    #[test]
    fn malformed_base64_rejected() {
        let v = &FIXTURES[0];
        assert_eq!(
            verify_sui_personal_message(v.address, "not-base64!!!", v.nonce.as_bytes()),
            Err(Error::InvalidSignature)
        );
    }

    #[test]
    fn malformed_signature_length_rejected() {
        let v = &FIXTURES[0];
        let mut sig = STANDARD.decode(v.signature).unwrap();
        sig.truncate(sig.len() - 1);
        assert_eq!(
            verify_sui_personal_message(v.address, &STANDARD.encode(&sig), v.nonce.as_bytes()),
            Err(Error::InvalidSignature)
        );
    }

    #[test]
    fn unknown_signature_flag_rejected() {
        let v = &FIXTURES[0];
        let mut sig = STANDARD.decode(v.signature).unwrap();
        sig[0] = 0xff;
        assert_eq!(
            verify_sui_personal_message(v.address, &STANDARD.encode(&sig), v.nonce.as_bytes()),
            Err(Error::InvalidSignature)
        );
    }

    #[test]
    fn address_parsing_accepts_lowercase_hex() {
        let addr = "0x0881c07520943bbf13989b92892093c1b50672156fa5f873c22892701cb2e207";
        let parsed = parse_address(addr).unwrap();
        assert_eq!(
            parsed,
            [
                0x08, 0x81, 0xc0, 0x75, 0x20, 0x94, 0x3b, 0xbf, 0x13, 0x98, 0x9b, 0x92, 0x89, 0x20,
                0x93, 0xc1, 0xb5, 0x06, 0x72, 0x15, 0x6f, 0xa5, 0xf8, 0x73, 0xc2, 0x28, 0x92, 0x70,
                0x1c, 0xb2, 0xe2, 0x07
            ]
        );
    }

    #[test]
    fn address_parsing_rejects_too_short() {
        assert!(parse_address("0x0881").is_err());
    }

    #[test]
    fn address_parsing_rejects_invalid_hex() {
        assert!(parse_address(
            "0xzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"
        )
        .is_err());
    }
}
