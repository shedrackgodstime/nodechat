use crate::backend::crypto;
use crate::contract::IdentityView;
use iroh::SecretKey;

pub fn generate_identity(display_name: &str) -> (IdentityView, SecretKey) {
    let secret_key = SecretKey::generate();
    let public_key = secret_key.public().to_string();
    let id = uuid::Uuid::new_v4().to_string();

    let identity = IdentityView {
        id,
        display_name: display_name.to_string(),
        public_key,
        created_at: chrono::Utc::now().timestamp(),
    };

    (identity, secret_key)
}

pub fn secret_key_to_hex(key: &SecretKey) -> String {
    hex::encode(key.to_bytes())
}

pub fn secret_key_from_hex(hex_str: &str) -> anyhow::Result<SecretKey> {
    let bytes = hex::decode(hex_str)?;
    let key: [u8; 32] = bytes.try_into().map_err(|_| anyhow::anyhow!("invalid key length"))?;
    Ok(SecretKey::from_bytes(&key))
}
