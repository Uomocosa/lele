use anyhow::Result;
use iroh::SecretKey;

pub fn get_secret_key_from_hex(hex: &str) -> Result<SecretKey> {
    let decoded_bytes = hex::decode(hex)?;
    assert_eq!(decoded_bytes.len(), 32);
    let mut array = [0_u8; 32];
    array.copy_from_slice(&decoded_bytes);
    Ok(SecretKey::from_bytes(&array))
}
