#[cfg(not(target_arch = "bpf"))]
use rand_core::{OsRng, CryptoRng, RngCore};
use {
    crate::encryption::elgamal::{
        CipherKey,
        ElGamalPubkey,
    },
    core::ops::{Mul},
    curve25519_dalek::{
        constants::{RISTRETTO_BASEPOINT_COMPRESSED, RISTRETTO_BASEPOINT_POINT},
        ristretto::{CompressedRistretto, RistrettoPoint},
        scalar::Scalar,
    },
    serde::{Deserialize, Serialize},
    sha3::Sha3_512,
    std::convert::TryInto,
    subtle::{Choice, ConstantTimeEq},
    zeroize::Zeroize,
};

/// Curve basepoints for which Pedersen commitment is defined over.
///
/// These points should be fixed for the entire system.
/// TODO: Consider setting these points as constants?
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PedersenBase {
    pub G: RistrettoPoint,
    pub H: RistrettoPoint,
}
/// Default PedersenBase. This is set arbitrarily for now, but it should be fixed
/// for the entire system.
///
/// `G` is a constant point in the curve25519_dalek library
/// `H` is the Sha3 hash of `G` interpretted as a RistrettoPoint
impl Default for PedersenBase {
    #[allow(non_snake_case)]
    fn default() -> PedersenBase {
        let G = RISTRETTO_BASEPOINT_POINT;
        let H =
            RistrettoPoint::hash_from_bytes::<Sha3_512>(RISTRETTO_BASEPOINT_COMPRESSED.as_bytes());

        PedersenBase { G, H }
    }
}

/// Handle for the Pedersen commitment scheme
pub struct Pedersen;
impl Pedersen {
    /// Given a number as input, the function returns a Pedersen commitment of
    /// the number and its corresponding opening.
    #[cfg(not(target_arch = "bpf"))]
    #[allow(clippy::new_ret_no_self)]
    pub fn new<T: Into<CipherKey>>(amount: T) -> (PedersenCommitment, PedersenOpening) {
        let open = PedersenOpening(Scalar::random(&mut OsRng));
        let comm = Pedersen::with(amount, &open);

        (comm, open)
    }

    /// Given a number and an opening as inputs, the function returns their
    /// Pedersen commitment.
    ///
    /// We use the high 8 bytes (63 bits) to indicate the 'correct' decoded value when reversing
    /// the ristretto -> jacobi -> elligator morphisms. Effectively, this function really should
    /// just be used for CipherKey's but for test compatibility...
    #[allow(non_snake_case)]
    pub fn with<T: Into<CipherKey>>(amount: T, open: &PedersenOpening) -> PedersenCommitment {
        let H = PedersenBase::default().H;

        let cipher_key: CipherKey = amount.into();

        let mut bytes = [0u8; 32];
        bytes[..24].copy_from_slice(&cipher_key.0);

        let r = open.get_scalar();

        let M = RistrettoPoint::elligator_ristretto_flavor(
            &curve25519_dalek::field::FieldElement::from_bytes(
                &bytes
                )
            );

        PedersenCommitment(M + H.mul(r))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Zeroize)]
#[zeroize(drop)]
pub struct PedersenOpening(pub(crate) Scalar);
impl PedersenOpening {
    pub fn get_scalar(&self) -> Scalar {
        self.0
    }

    #[cfg(not(target_arch = "bpf"))]
    pub fn random<T: RngCore + CryptoRng>(rng: &mut T) -> Self {
        PedersenOpening(Scalar::random(rng))
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<PedersenOpening> {
        match bytes.try_into() {
            Ok(bytes) => Scalar::from_canonical_bytes(bytes).map(PedersenOpening),
            _ => None,
        }
    }
}
impl Eq for PedersenOpening {}
impl PartialEq for PedersenOpening {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1u8
    }
}
impl ConstantTimeEq for PedersenOpening {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl Default for PedersenOpening {
    fn default() -> Self {
        PedersenOpening(Scalar::default())
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PedersenCommitment(pub(crate) RistrettoPoint);
impl PedersenCommitment {
    pub fn get_point(&self) -> RistrettoPoint {
        self.0
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.compress().to_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<PedersenCommitment> {
        Some(PedersenCommitment(
            CompressedRistretto::from_slice(bytes).decompress()?,
        ))
    }
}


/// Decryption handle for Pedersen commitment.
///
/// A decryption handle can be combined with Pedersen commitments to form an
/// ElGamal ciphertext.
#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PedersenDecryptHandle(pub(crate) RistrettoPoint);
impl PedersenDecryptHandle {
    pub fn new(pk: &ElGamalPubkey, open: &PedersenOpening) -> Self {
        Self(pk.get_point() * open.get_scalar())
    }

    pub fn get_point(&self) -> RistrettoPoint {
        self.0
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.compress().to_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<PedersenDecryptHandle> {
        Some(PedersenDecryptHandle(
            CompressedRistretto::from_slice(bytes).decompress()?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_bytes() {
        let amt: u64 = 77;
        let (comm, _) = Pedersen::new(amt);

        let encoded = comm.to_bytes();
        let decoded = PedersenCommitment::from_bytes(&encoded).unwrap();

        assert_eq!(comm, decoded);
    }

    #[test]
    fn test_opening_bytes() {
        let open = PedersenOpening(Scalar::random(&mut OsRng));

        let encoded = open.to_bytes();
        let decoded = PedersenOpening::from_bytes(&encoded).unwrap();

        assert_eq!(open, decoded);
    }

    #[test]
    fn test_decrypt_handle_bytes() {
        let handle = PedersenDecryptHandle(RistrettoPoint::default());

        let encoded = handle.to_bytes();
        let decoded = PedersenDecryptHandle::from_bytes(&encoded).unwrap();

        assert_eq!(handle, decoded);
    }

    #[test]
    fn test_serde_commitment() {
        let amt: u64 = 77;
        let (comm, _) = Pedersen::new(amt);

        let encoded = bincode::serialize(&comm).unwrap();
        let decoded: PedersenCommitment = bincode::deserialize(&encoded).unwrap();

        assert_eq!(comm, decoded);
    }

    #[test]
    fn test_serde_opening() {
        let open = PedersenOpening(Scalar::random(&mut OsRng));

        let encoded = bincode::serialize(&open).unwrap();
        let decoded: PedersenOpening = bincode::deserialize(&encoded).unwrap();

        assert_eq!(open, decoded);
    }

    #[test]
    fn test_serde_decrypt_handle() {
        let handle = PedersenDecryptHandle(RistrettoPoint::default());

        let encoded = bincode::serialize(&handle).unwrap();
        let decoded: PedersenDecryptHandle = bincode::deserialize(&encoded).unwrap();

        assert_eq!(handle, decoded);
    }
}
