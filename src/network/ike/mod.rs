use rand::SecureRandom;
use ring::{hmac, rand};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub mod crypto;
pub mod session;
pub mod tunnels;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IKESession {
    pub local_spi: u64,
    pub remote_spi: u64,
    pub shared_secret: Vec<u8>,
    pub encryption_key: Vec<u8>,
    pub authentication_key: Vec<u8>,
    pub state: IKEState,
    pub peer_addr: SocketAddr,
    pub dh_group: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IKEState {
    Initial,
    SaInit,
    Auth,
    Established,
    Rekeying,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IKEMessage {
    pub initiator_spi: u64,
    pub responder_spi: u64,
    pub next_payload: u8,
    pub version: u8,
    pub exchange_type: ExchangeType,
    pub flags: u8,
    pub message_id: u32,
    pub length: u32,
    pub payloads: Vec<IKEPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExchangeType {
    IkeSaInit = 34,
    IkeAuth = 35,
    CreateChildSa = 36,
    Informational = 37,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IKEPayload {
    SA(SAPayload),
    KeyExchange(KeyExchangePayload),
    Nonce(NoncePayload),
    Notification(NotificationPayload),
    Authentication(AuthPayload),
    Unknown { payload_type: u8, data: Vec<u8> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SAPayload {
    pub proposals: Vec<SAProposal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SAProposal {
    pub proposal_num: u8,
    pub protocol_id: u8,
    pub spi: Vec<u8>,
    pub transforms: Vec<Transform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub transform_type: u8,
    pub transform_id: u16,
    pub attributes: Vec<TransformAttribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformAttribute {
    pub attribute_type: u16,
    pub attribute_value: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExchangePayload {
    pub dh_group: u16,
    pub key_exchange_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoncePayload {
    pub nonce_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub protocol_id: u8,
    pub spi_size: u8,
    pub notify_message_type: u16,
    pub spi: Vec<u8>,
    pub notification_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthPayload {
    pub auth_method: u8,
    pub auth_data: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum IKEError {
    #[error("Crypto error: {0}")]
    Crypto(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Network error: {0}")]
    Network(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}

impl IKESession {
    pub fn new(peer_addr: SocketAddr, dh_group: u8) -> Result<Self, IKEError> {
        let rng = rand::SystemRandom::new();
        let mut local_spi = [0u8; 8];
        rng.fill(&mut local_spi)
            .map_err(|e| IKEError::Crypto(format!("RNG error: {:?}", e)))?;

        Ok(IKESession {
            local_spi: u64::from_be_bytes(local_spi),
            remote_spi: 0,
            shared_secret: Vec::new(),
            encryption_key: Vec::new(),
            authentication_key: Vec::new(),
            state: IKEState::Initial,
            peer_addr,
            dh_group,
        })
    }

    pub async fn establish_tunnel(&mut self, psk: &[u8]) -> Result<(), IKEError> {
        tracing::info!("Establishing IKE tunnel to {}", self.peer_addr);

        // Phase 1: IKE_SA_INIT exchange
        self.perform_sa_init().await?;

        // Phase 2: IKE_AUTH exchange
        self.perform_auth(psk).await?;

        self.state = IKEState::Established;
        tracing::info!("IKE tunnel established successfully");

        Ok(())
    }

    async fn perform_sa_init(&mut self) -> Result<(), IKEError> {
        tracing::debug!("Performing IKE_SA_INIT exchange");

        self.state = IKEState::SaInit;

        // Generate DH key pair
        let (public_key, _private_key) = self.generate_dh_keypair()?;

        // Create SA proposal
        let sa_payload = self.create_sa_proposal();

        // Generate nonce
        let nonce = self.generate_nonce()?;

        // Create IKE_SA_INIT request
        let _init_message = IKEMessage {
            initiator_spi: self.local_spi,
            responder_spi: 0,
            next_payload: 0,
            version: 0x20, // IKEv2
            exchange_type: ExchangeType::IkeSaInit,
            flags: 0x08, // Initiator flag
            message_id: 0,
            length: 0, // Will be calculated
            payloads: vec![
                IKEPayload::SA(sa_payload),
                IKEPayload::KeyExchange(KeyExchangePayload {
                    dh_group: self.dh_group as u16,
                    key_exchange_data: public_key,
                }),
                IKEPayload::Nonce(NoncePayload { nonce_data: nonce }),
            ],
        };

        // For now, just simulate the exchange
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Simulate receiving response and computing shared secret
        self.shared_secret = vec![0x42; 32]; // Placeholder
        self.derive_keys()?;

        Ok(())
    }

    async fn perform_auth(&mut self, psk: &[u8]) -> Result<(), IKEError> {
        tracing::debug!("Performing IKE_AUTH exchange");

        self.state = IKEState::Auth;

        // Create authentication data
        let auth_data = self.create_auth_data(psk)?;

        let _auth_message = IKEMessage {
            initiator_spi: self.local_spi,
            responder_spi: self.remote_spi,
            next_payload: 0,
            version: 0x20,
            exchange_type: ExchangeType::IkeAuth,
            flags: 0x08,
            message_id: 1,
            length: 0,
            payloads: vec![IKEPayload::Authentication(AuthPayload {
                auth_method: 2, // PSK
                auth_data,
            })],
        };

        // Simulate the exchange
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        Ok(())
    }

    fn create_sa_proposal(&self) -> SAPayload {
        SAPayload {
            proposals: vec![SAProposal {
                proposal_num: 1,
                protocol_id: 1, // IKE
                spi: Vec::new(),
                transforms: vec![
                    Transform {
                        transform_type: 1, // Encryption
                        transform_id: 20,  // AES-256-GCM
                        attributes: vec![],
                    },
                    Transform {
                        transform_type: 2, // PRF
                        transform_id: 5,   // HMAC-SHA256
                        attributes: vec![],
                    },
                    Transform {
                        transform_type: 3, // Integrity
                        transform_id: 12,  // AUTH_HMAC_SHA2_256_128
                        attributes: vec![],
                    },
                    Transform {
                        transform_type: 4, // DH Group
                        transform_id: self.dh_group as u16,
                        attributes: vec![],
                    },
                ],
            }],
        }
    }

    fn generate_nonce(&self) -> Result<Vec<u8>, IKEError> {
        let rng = rand::SystemRandom::new();
        let mut nonce = vec![0u8; 32];
        rng.fill(&mut nonce)
            .map_err(|e| IKEError::Crypto(format!("Nonce generation failed: {:?}", e)))?;
        Ok(nonce)
    }

    fn generate_dh_keypair(&self) -> Result<(Vec<u8>, Vec<u8>), IKEError> {
        // Simplified DH key generation - in a real implementation,
        // this would use proper DH groups (14, 19, 20, etc.)
        let rng = rand::SystemRandom::new();

        let mut private_key = vec![0u8; 32];
        let mut public_key = vec![0u8; 32];

        rng.fill(&mut private_key)
            .map_err(|e| IKEError::Crypto(format!("Private key generation failed: {:?}", e)))?;
        rng.fill(&mut public_key)
            .map_err(|e| IKEError::Crypto(format!("Public key generation failed: {:?}", e)))?;

        Ok((public_key, private_key))
    }

    fn derive_keys(&mut self) -> Result<(), IKEError> {
        // Simplified key derivation - in production, use proper HKDF
        let key_material = self.shared_secret.clone();

        // Derive 32-byte encryption key
        let mut encryption_key = vec![0u8; 32];
        for (i, byte) in key_material.iter().cycle().enumerate().take(32) {
            encryption_key[i] = *byte ^ (i as u8);
        }

        // Derive 32-byte authentication key
        let mut auth_key = vec![0u8; 32];
        for (i, byte) in key_material.iter().cycle().enumerate().take(32) {
            auth_key[i] = *byte ^ ((i + 1) as u8);
        }

        self.encryption_key = encryption_key;
        self.authentication_key = auth_key;

        Ok(())
    }

    fn create_auth_data(&self, psk: &[u8]) -> Result<Vec<u8>, IKEError> {
        let key = hmac::Key::new(hmac::HMAC_SHA256, psk);
        let auth_data = hmac::sign(&key, &self.shared_secret);
        Ok(auth_data.as_ref().to_vec())
    }

    pub fn is_established(&self) -> bool {
        matches!(self.state, IKEState::Established)
    }

    pub async fn close(&mut self) -> Result<(), IKEError> {
        self.state = IKEState::Deleted;
        tracing::info!("IKE session closed");
        Ok(())
    }
}
