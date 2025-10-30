use std::io::{Error, ErrorKind};
use aes_gcm::{Aes256Gcm};
use aes_gcm::aead::{AeadCore, Aead, KeyInit};

#[derive(Debug)]
pub enum CypherError {
    DecryptionError,
    EncryptError,
    GenerateNonceError,
}

impl From<CypherError> for Error {
    fn from(error: CypherError) -> Error {
        match error {
            CypherError::DecryptionError => Error::new(ErrorKind::Other, "Decryption error"),
            CypherError::EncryptError => Error::new(ErrorKind::Other, "Encryption error"),
            CypherError::GenerateNonceError => Error::new(ErrorKind::Other, "Generate nonce error"),
        }
    }
}

pub struct Cypher {
    cypher: Aes256Gcm,
}

impl Cypher {
    pub fn new(key: &[u8; 32]) -> Self {
        let cypher = Aes256Gcm::new(key.into());
        Self { cypher }
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CypherError> {
        let nonce = Aes256Gcm::generate_nonce().map_err(|_| CypherError::GenerateNonceError)?;
        let encrypted_data = self.cypher.encrypt(&nonce, data).map_err(|_| CypherError::EncryptError)?;
        Ok([nonce.to_vec(), encrypted_data].concat())
    }

    pub fn decrypt(&self, nonce_bytes: &[u8;12], encrypted_data: &[u8]) -> Result<Vec<u8>, CypherError> {
        let nonce = nonce_bytes.into();
        Ok(self.cypher.decrypt(nonce, encrypted_data).map_err(|_| CypherError::DecryptionError)?)
    }
}