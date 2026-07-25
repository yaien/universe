use aes_gcm::aead::{Aead, Generate, KeyInit, Nonce};
use aes_gcm::{Aes128Gcm, Key, KeySizeUser};
use anyhow::Result;
use serde::Serialize;

pub fn generate_key_hex<T: KeySizeUser>() -> String {
    let key = Key::<T>::generate();
    hex::encode(key)
}

pub fn encrypt<K: Serialize>(hx: &str, s: &K) -> Result<String> {
    let plain = hex::decode(hx)?;
    let key = Key::<Aes128Gcm>::try_from(&plain[..])?;
    let cipher = Aes128Gcm::new(&key);
    let nonce = Nonce::<Aes128Gcm>::generate();
    let plaintext = serde_json::to_string(s)?;
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes())?;
    let nonce_str = hex::encode(nonce.to_vec());
    let ciphertext_str = hex::encode(ciphertext);
    Ok(format!("{nonce_str}.{ciphertext_str}"))
}

pub fn decrypt<K>(hx: &str, encrypted: &str) -> Result<K>
where
    K: for<'de> serde::Deserialize<'de>,
{
    let decoded = hex::decode(hx)?;
    let key = Key::<Aes128Gcm>::try_from(&decoded[..])?;
    let cipher = Aes128Gcm::new(&key);
    let (nonce, ciphertext) = encrypted
        .split_once('.')
        .ok_or(anyhow::anyhow!("invalid ciphertext"))?;
    let nonce = hex::decode(nonce)?;
    let nonce = Nonce::<Aes128Gcm>::try_from(&nonce[..])?;
    let ciphertext = hex::decode(ciphertext)?;
    let plaintext = cipher.decrypt(&nonce, &ciphertext[..])?;

    serde_json::from_slice(&plaintext[..]).map_err(|e| anyhow::anyhow!(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() -> Result<()> {
        let key = generate_key_hex::<Aes128Gcm>();
        let original = serde_json::json!({ "test": 42 });
        let encrypted = encrypt(&key, &original)?;
        let decrypted = decrypt::<serde_json::Value>(&key, &encrypted)?;
        assert_eq!(original, decrypted);
        Ok(())
    }
}
