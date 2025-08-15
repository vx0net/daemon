use ring::{aead, hmac, rand};
use rand::SecureRandom;
use crate::network::ike::IKEError;

pub struct IKECrypto {
    pub encryption_algorithm: EncryptionAlgorithm,
    pub hash_algorithm: HashAlgorithm,
    pub dh_group: DHGroup,
}

#[derive(Debug, Clone)]
pub enum EncryptionAlgorithm {
    AES128,
    AES256,
    ChaCha20Poly1305,
}

#[derive(Debug, Clone)]
pub enum HashAlgorithm {
    SHA256,
    SHA384,
    SHA512,
}

#[derive(Debug, Clone)]
pub enum DHGroup {
    Group14, // 2048-bit MODP
    Group19, // 256-bit Random ECP
    Group20, // 384-bit Random ECP
}

impl IKECrypto {
    pub fn new() -> Self {
        IKECrypto {
            encryption_algorithm: EncryptionAlgorithm::AES256,
            hash_algorithm: HashAlgorithm::SHA256,
            dh_group: DHGroup::Group14,
        }
    }

    pub fn encrypt(&self, key: &[u8], plaintext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, IKEError> {
        match self.encryption_algorithm {
            EncryptionAlgorithm::AES256 => {
                self.aes256_gcm_encrypt(key, plaintext, nonce)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                self.chacha20_poly1305_encrypt(key, plaintext, nonce)
            }
            _ => Err(IKEError::Crypto("Unsupported encryption algorithm".to_string())),
        }
    }

    pub fn decrypt(&self, key: &[u8], ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, IKEError> {
        match self.encryption_algorithm {
            EncryptionAlgorithm::AES256 => {
                self.aes256_gcm_decrypt(key, ciphertext, nonce)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                self.chacha20_poly1305_decrypt(key, ciphertext, nonce)
            }
            _ => Err(IKEError::Crypto("Unsupported encryption algorithm".to_string())),
        }
    }

    fn aes256_gcm_encrypt(&self, key: &[u8], plaintext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, IKEError> {
        if key.len() != 32 {
            return Err(IKEError::Crypto("Invalid key size for AES-256".to_string()));
        }

        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, key)
            .map_err(|_| IKEError::Crypto("Failed to create AES key".to_string()))?;
        
        let sealing_key = aead::LessSafeKey::new(unbound_key);
        
        let mut in_out = plaintext.to_vec();
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce)
            .map_err(|_| IKEError::Crypto("Invalid nonce".to_string()))?;
        
        sealing_key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| IKEError::Crypto("Encryption failed".to_string()))?;
        
        Ok(in_out)
    }

    fn aes256_gcm_decrypt(&self, key: &[u8], ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, IKEError> {
        if key.len() != 32 {
            return Err(IKEError::Crypto("Invalid key size for AES-256".to_string()));
        }

        let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, key)
            .map_err(|_| IKEError::Crypto("Failed to create AES key".to_string()))?;
        
        let opening_key = aead::LessSafeKey::new(unbound_key);
        
        let mut in_out = ciphertext.to_vec();
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce)
            .map_err(|_| IKEError::Crypto("Invalid nonce".to_string()))?;
        
        let plaintext = opening_key.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| IKEError::Crypto("Decryption failed".to_string()))?;
        
        Ok(plaintext.to_vec())
    }

    fn chacha20_poly1305_encrypt(&self, key: &[u8], plaintext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, IKEError> {
        if key.len() != 32 {
            return Err(IKEError::Crypto("Invalid key size for ChaCha20-Poly1305".to_string()));
        }

        let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key)
            .map_err(|_| IKEError::Crypto("Failed to create ChaCha20 key".to_string()))?;
        
        let sealing_key = aead::LessSafeKey::new(unbound_key);
        
        let mut in_out = plaintext.to_vec();
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce)
            .map_err(|_| IKEError::Crypto("Invalid nonce".to_string()))?;
        
        sealing_key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| IKEError::Crypto("Encryption failed".to_string()))?;
        
        Ok(in_out)
    }

    fn chacha20_poly1305_decrypt(&self, key: &[u8], ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, IKEError> {
        if key.len() != 32 {
            return Err(IKEError::Crypto("Invalid key size for ChaCha20-Poly1305".to_string()));
        }

        let unbound_key = aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key)
            .map_err(|_| IKEError::Crypto("Failed to create ChaCha20 key".to_string()))?;
        
        let opening_key = aead::LessSafeKey::new(unbound_key);
        
        let mut in_out = ciphertext.to_vec();
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce)
            .map_err(|_| IKEError::Crypto("Invalid nonce".to_string()))?;
        
        let plaintext = opening_key.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| IKEError::Crypto("Decryption failed".to_string()))?;
        
        Ok(plaintext.to_vec())
    }

    pub fn generate_nonce(&self, size: usize) -> Result<Vec<u8>, IKEError> {
        let rng = rand::SystemRandom::new();
        let mut nonce = vec![0u8; size];
        rng.fill(&mut nonce)
            .map_err(|e| IKEError::Crypto(format!("Nonce generation failed: {:?}", e)))?;
        Ok(nonce)
    }

    pub fn hmac_sign(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>, IKEError> {
        let hmac_key = match self.hash_algorithm {
            HashAlgorithm::SHA256 => hmac::Key::new(hmac::HMAC_SHA256, key),
            HashAlgorithm::SHA384 => hmac::Key::new(hmac::HMAC_SHA384, key),
            HashAlgorithm::SHA512 => hmac::Key::new(hmac::HMAC_SHA512, key),
        };

        let signature = hmac::sign(&hmac_key, data);
        Ok(signature.as_ref().to_vec())
    }

    pub fn hmac_verify(&self, key: &[u8], data: &[u8], signature: &[u8]) -> Result<bool, IKEError> {
        let hmac_key = match self.hash_algorithm {
            HashAlgorithm::SHA256 => hmac::Key::new(hmac::HMAC_SHA256, key),
            HashAlgorithm::SHA384 => hmac::Key::new(hmac::HMAC_SHA384, key),
            HashAlgorithm::SHA512 => hmac::Key::new(hmac::HMAC_SHA512, key),
        };

        match hmac::verify(&hmac_key, data, signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

impl Default for IKECrypto {
    fn default() -> Self {
        Self::new()
    }
}