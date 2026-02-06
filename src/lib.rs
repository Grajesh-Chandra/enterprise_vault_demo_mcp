/*! Common componets between the client and the Service
*/

use affinidi_tdk::{
    did_peer::DIDPeerKeys,
    dids::{DID, KeyType},
    secrets_resolver::secrets::Secret,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

// CLI Color codes
pub const CLI_BLUE: u8 = 69; // Use for general information
pub const CLI_GREEN: u8 = 34; // Use for Successful text
pub const CLI_RED: u8 = 9; // Use for Error messages
pub const CLI_ORANGE: u8 = 214; // Use for cautionary data

pub const DEFAULT_MEDIATOR: &str = "did:web:mediator-nlb.storm.ws:mediator:v1:.well-known";

/// Password Request Struct
#[derive(Deserialize, Serialize)]
pub struct PasswordRequest {
    pub key: String,
}

/// Password Response Enum
#[derive(Deserialize, Serialize)]
pub enum PasswordResponse {
    /// Password was not found for the requested key
    NotFound,

    /// Password was found for the requested key
    Found { key: String, password: String },
}

/// Generates a did:peer address for this service
pub fn generate_did(mediator_did: &str) -> Result<(String, Vec<Secret>)> {
    Ok(DID::generate_did_peer(
        vec![
            (DIDPeerKeys::Verification, KeyType::Ed25519),
            (DIDPeerKeys::Encryption, KeyType::Secp256k1),
        ],
        Some(mediator_did.to_string()),
    )?)
}
