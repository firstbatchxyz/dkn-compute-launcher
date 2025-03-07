use eyre::Context;
use libsecp256k1::{Message, PublicKey, SecretKey};
use sha3::{Digest, Keccak256};

/// Given a secp256k1 public key, finds the corresponding Ethereum address.
///
/// Internally, the public key is serialized in uncompressed format at 65 bytes (0x04 || x || y),
/// and then (x || y) is hashed using Keccak256. The last 20 bytes of this hash is taken as the address.
///
/// The returned string is a hex-encoded lowercase string of the address.
#[inline]
pub fn public_key_to_address(public_key: &libsecp256k1::PublicKey) -> String {
    let public_key_xy = &public_key.serialize()[1..];
    let digest = Message::parse(&Keccak256::digest(public_key_xy).into());
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&digest.serialize()[12..32]);

    // we dont expect to panic here at all
    hex::encode(&addr)
}

/// Given a hexadecimal string representing a secp256k1 secret key, returns the corresponding secret key, public key, and address.
#[inline]
pub fn parse_key_to_account(key: &str) -> eyre::Result<(SecretKey, PublicKey, String)> {
    let parsed_secret = hex::decode(key).wrap_err("could not parse secret key")?;
    let secret_key = libsecp256k1::SecretKey::parse_slice(&parsed_secret)
        .wrap_err("could not parse secret key")?;
    let public_key = libsecp256k1::PublicKey::from_secret_key(&secret_key);
    let address = public_key_to_address(&public_key);

    Ok((secret_key, public_key, address))
}
