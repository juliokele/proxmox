//! Auth key handling.

use anyhow::{bail, format_err, Error};
use openssl::ec::{EcGroup, EcKey};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{HasPublic, PKey, PKeyRef, Private, Public};
use openssl::rsa::Rsa;
use openssl::sign::{Signer, Verifier};

/// A private auth key used for API ticket signing and verification.
pub struct PrivateKey {
    pub(crate) key: PKey<Private>,
}

/// A private auth key used for API ticket verification.
pub struct PublicKey {
    pub(crate) key: PKey<Public>,
}

impl PrivateKey {
    /// Generate a new RSA auth key.
    pub fn generate_rsa() -> Result<Self, Error> {
        let rsa =
            Rsa::generate(4096).map_err(|err| format_err!("failed to generate rsa key - {err}"))?;
        Ok(Self {
            key: PKey::from_rsa(rsa)
                .map_err(|err| format_err!("failed to get PKey for rsa key - {err}"))?,
        })
    }

    /// Generate a new EC auth key.
    pub fn generate_ec() -> Result<Self, Error> {
        let nid = Nid::X9_62_PRIME256V1;
        let group = EcGroup::from_curve_name(nid)
            .map_err(|err| format_err!("failed to get P-256 group - {err}"))?;
        let ec = EcKey::generate(&group)
            .map_err(|err| format_err!("failed to generate EC key for testing - {err}"))?;
        Ok(Self {
            key: PKey::from_ec_key(ec)
                .map_err(|err| format_err!("failed to get PKey for EC key - {err}"))?,
        })
    }

    pub fn from_pem(data: &[u8]) -> Result<Self, Error> {
        let key = PKey::private_key_from_pem(data)
            .map_err(|err| format_err!("failed to decode private key from PEM - {err}"))?;
        Ok(Self { key })
    }

    /// Get the PEM formatted private key *unencrypted*.
    pub fn private_key_to_pem(&self) -> Result<Vec<u8>, Error> {
        // No PKCS#8 for legacy reasons:
        if let Ok(rsa) = self.key.rsa() {
            return rsa
                .private_key_to_pem()
                .map_err(|err| format_err!("failed to encode rsa private key as PEM - {err}"));
        }

        if let Ok(ec) = self.key.ec_key() {
            return ec
                .private_key_to_pem()
                .map_err(|err| format_err!("failed to encode ec private key as PEM - {err}"));
        }

        bail!("unexpected key data")
    }

    /// Get the PEM formatted public key.
    pub fn public_key_to_pem(&self) -> Result<Vec<u8>, Error> {
        // No PKCS#8 for legacy reasons:
        if let Ok(rsa) = self.key.rsa() {
            return rsa
                .public_key_to_pem()
                .map_err(|err| format_err!("failed to encode rsa public key as PEM - {err}"));
        }

        if let Ok(ec) = self.key.ec_key() {
            return ec
                .public_key_to_pem()
                .map_err(|err| format_err!("failed to encode ec public key as PEM - {err}"));
        }

        bail!("unexpected key data")
    }

    /// Get the public key.
    pub fn public_key(&self) -> Result<PublicKey, Error> {
        PublicKey::from_pem(&self.public_key_to_pem()?)
    }
}

impl From<PKey<Private>> for PrivateKey {
    fn from(key: PKey<Private>) -> Self {
        Self { key }
    }
}

impl PublicKey {
    pub fn from_pem(data: &[u8]) -> Result<Self, Error> {
        let key = PKey::public_key_from_pem(data)
            .map_err(|err| format_err!("failed to decode public key from PEM - {err}"))?;
        Ok(Self { key })
    }

    /// Get the PEM formatted public key.
    pub fn public_key_to_pem(&self) -> Result<Vec<u8>, Error> {
        // No PKCS#8 for legacy reasons:
        if let Ok(rsa) = self.key.rsa() {
            return rsa
                .public_key_to_pem()
                .map_err(|err| format_err!("failed to encode rsa public key as PEM - {err}"));
        }

        if let Ok(ec) = self.key.ec_key() {
            return ec
                .public_key_to_pem()
                .map_err(|err| format_err!("failed to encode ec public key as PEM - {err}"));
        }

        bail!("unexpected key data")
    }
}

impl From<PKey<Public>> for PublicKey {
    fn from(key: PKey<Public>) -> Self {
        Self { key }
    }
}

/// A key ring for authentication.
///
/// This holds one active signing key for new tickets, and optionally multiple public keys for
/// verifying them in order to support key rollover.
pub struct Keyring {
    signing_key: Option<PrivateKey>,
    public_keys: Vec<PublicKey>,
}

impl Keyring {
    pub fn generate_new_rsa() -> Result<Self, Error> {
        PrivateKey::generate_rsa().map(Self::with_private_key)
    }

    pub fn generate_new_ec() -> Result<Self, Error> {
        PrivateKey::generate_ec().map(Self::with_private_key)
    }

    pub fn new() -> Self {
        Self {
            signing_key: None,
            public_keys: Vec::new(),
        }
    }

    pub fn with_public_key(key: PublicKey) -> Self {
        Self {
            signing_key: None,
            public_keys: vec![key],
        }
    }

    pub fn with_private_key(key: PrivateKey) -> Self {
        Self {
            signing_key: Some(key),
            public_keys: Vec::new(),
        }
    }

    pub fn add_public_key(&mut self, key: PublicKey) {
        self.public_keys.push(key);
    }

    pub fn verify(
        &self,
        digest: MessageDigest,
        signature: &[u8],
        data: &[u8],
    ) -> Result<bool, Error> {
        fn verify_with<P: HasPublic>(
            key: &PKeyRef<P>,
            digest: MessageDigest,
            signature: &[u8],
            data: &[u8],
        ) -> Result<bool, Error> {
            Verifier::new(digest, key)
                .map_err(|err| format_err!("failed to create openssl verifier - {err}"))?
                .verify_oneshot(signature, data)
                .map_err(|err| format_err!("openssl error verifying data - {err}"))
        }

        if let Some(key) = &self.signing_key {
            if verify_with(&key.key, digest, signature, data)? {
                return Ok(true);
            }
        }

        for key in &self.public_keys {
            if verify_with(&key.key, digest, signature, data)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub(crate) fn signer(&self, digest: MessageDigest) -> Result<Signer, Error> {
        Signer::new(
            digest,
            &self
                .signing_key
                .as_ref()
                .ok_or_else(|| format_err!("no private key available for signing"))?
                .key,
        )
        .map_err(|err| format_err!("failed to create openssl signer - {err}"))
    }
}
