#[cfg(not(target_arch = "bpf"))]
use {
    arrayref::{array_ref, array_refs},
    curve25519_dalek::{
        ristretto::{RistrettoPoint},
        traits::{MultiscalarMul, VartimeMultiscalarMul},
    },
    rand::rngs::OsRng,
    spl_zk_token_sdk::encryption::{
        elgamal::{ElGamalCiphertext, ElGamalKeypair, ElGamalPubkey},
        pedersen::{PedersenBase, PedersenOpening},
    },
};
use {
    curve25519_dalek::{
        ristretto::{CompressedRistretto},
        scalar::Scalar,
        traits::{IsIdentity},
    },
    merlin::Transcript,
    thiserror::Error,
};

#[derive(Error, Clone, Debug, Eq, PartialEq)]
pub enum ProofError {
    #[error("proof failed to verify")]
    VerificationError,
    #[error("malformed proof")]
    FormatError,
    #[error("number of blinding factors do not match the number of values")]
    WrongNumBlindingFactors,
    #[error("attempted to create a proof with bitsize other than \\(8\\), \\(16\\), \\(32\\), or \\(64\\)")]
    InvalidBitsize,
    #[error("insufficient generators for the proof")]
    InvalidGeneratorsLength,
    #[error(
        "`zk_token_elgamal::pod::ElGamalCiphertext` contains invalid ElGamalCiphertext ciphertext"
    )]
    InconsistentCTData,
}

pub trait TranscriptProtocol {
    /// Append a domain separator for an `n`-bit rangeproof for ElGamalKeypair
    /// ciphertext using a decryption key
    fn rangeproof_from_key_domain_sep(&mut self, n: u64);

    /// Append a domain separator for an `n`-bit rangeproof for ElGamalKeypair
    /// ciphertext using an opening
    fn rangeproof_from_opening_domain_sep(&mut self, n: u64);

    /// Append a domain separator for a length-`n` inner product proof.
    fn innerproduct_domain_sep(&mut self, n: u64);

    /// Append a domain separator for close account proof.
    fn close_account_proof_domain_sep(&mut self);

    /// Append a domain separator for update account public key proof.
    fn update_account_public_key_proof_domain_sep(&mut self);

    /// Append a domain separator for withdraw proof.
    fn withdraw_proof_domain_sep(&mut self);

    /// Append a domain separator for transfer proof.
    fn transfer_proof_domain_sep(&mut self);

    /// Append a `scalar` with the given `label`.
    fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar);

    /// Append a `point` with the given `label`.
    fn append_point(&mut self, label: &'static [u8], point: &CompressedRistretto);

    /// Check that a point is not the identity, then append it to the
    /// transcript.  Otherwise, return an error.
    fn validate_and_append_point(
        &mut self,
        label: &'static [u8],
        point: &CompressedRistretto,
    ) -> Result<(), ProofError>;

    /// Compute a `label`ed challenge variable.
    fn challenge_scalar(&mut self, label: &'static [u8]) -> Scalar;
}

impl TranscriptProtocol for Transcript {
    fn rangeproof_from_key_domain_sep(&mut self, n: u64) {
        self.append_message(b"dom-sep", b"rangeproof from opening v1");
        self.append_u64(b"n", n);
    }

    fn rangeproof_from_opening_domain_sep(&mut self, n: u64) {
        self.append_message(b"dom-sep", b"rangeproof from opening v1");
        self.append_u64(b"n", n);
    }

    fn innerproduct_domain_sep(&mut self, n: u64) {
        self.append_message(b"dom-sep", b"ipp v1");
        self.append_u64(b"n", n);
    }

    fn close_account_proof_domain_sep(&mut self) {
        self.append_message(b"dom-sep", b"CloseAccountProof");
    }

    fn update_account_public_key_proof_domain_sep(&mut self) {
        self.append_message(b"dom-sep", b"UpdateAccountPublicKeyProof");
    }

    fn withdraw_proof_domain_sep(&mut self) {
        self.append_message(b"dom-sep", b"WithdrawProof");
    }

    fn transfer_proof_domain_sep(&mut self) {
        self.append_message(b"dom-sep", b"TransferProof");
    }

    fn append_scalar(&mut self, label: &'static [u8], scalar: &Scalar) {
        self.append_message(label, scalar.as_bytes());
    }

    fn append_point(&mut self, label: &'static [u8], point: &CompressedRistretto) {
        self.append_message(label, point.as_bytes());
    }

    fn validate_and_append_point(
        &mut self,
        label: &'static [u8],
        point: &CompressedRistretto,
    ) -> Result<(), ProofError> {
        if point.is_identity() {
            Err(ProofError::VerificationError)
        } else {
            self.append_message(label, point.as_bytes());
            Ok(())
        }
    }

    fn challenge_scalar(&mut self, label: &'static [u8]) -> Scalar {
        let mut buf = [0u8; 64];
        self.challenge_bytes(label, &mut buf);

        Scalar::from_bytes_mod_order_wide(&buf)
    }
}

#[allow(non_snake_case)]
#[derive(Clone)]
pub struct EqualityProof {
    pub Y_0: CompressedRistretto,
    pub Y_1: CompressedRistretto,
    pub Y_2: CompressedRistretto,
    pub sh_1: Scalar,
    pub rh_2: Scalar,
}

#[allow(non_snake_case)]
#[cfg(not(target_arch = "bpf"))]
impl EqualityProof {
    pub fn new(
        src_keypair: &ElGamalKeypair,
        dst_pubkey: &ElGamalPubkey,
        src_ciphertext: &ElGamalCiphertext,
        dst_opening: &PedersenOpening,
        transcript: &mut Transcript,
    ) -> Self {
        // extract the relevant scalar and Ristretto points from the inputs
        let H = PedersenBase::default().H;

        let P1_EG = src_keypair.public.get_point();
        let P2_EG = dst_pubkey.get_point();
        let D1_EG = src_ciphertext.decrypt_handle.get_point();

        let s_1 = src_keypair.secret.get_scalar();
        let r_2 = dst_opening.get_scalar();

        // generate random masking factors that also serves as a nonce
        let b_1 = Scalar::random(&mut OsRng);
        let b_2 = Scalar::random(&mut OsRng);

        let Y_0 = (b_1 * P1_EG).compress();
        let Y_1 = (b_2 * P2_EG).compress();
        let Y_2 = RistrettoPoint::multiscalar_mul(vec![b_1, -b_2], vec![D1_EG, H]).compress();

        // record masking factors in transcript
        transcript.append_point(b"Y_0", &Y_0);
        transcript.append_point(b"Y_1", &Y_1);
        transcript.append_point(b"Y_2", &Y_2);

        let c = transcript.challenge_scalar(b"c");
        transcript.challenge_scalar(b"w");

        // compute the masked values
        let sh_1 = c * s_1 + b_1;
        let rh_2 = c * r_2 + b_2;

        EqualityProof {
            Y_0,
            Y_1,
            Y_2,
            sh_1,
            rh_2,
        }
    }

    pub fn verify(
        self,
        src_pubkey: &ElGamalPubkey,
        dst_pubkey: &ElGamalPubkey,
        src_ciphertext: &ElGamalCiphertext,
        dst_ciphertext: &ElGamalCiphertext,
        transcript: &mut Transcript,
    ) -> Result<(), ProofError> {
        // extract the relevant scalar and Ristretto points from the inputs
        let H = PedersenBase::default().H;

        let P1_EG = src_pubkey.get_point();
        let P2_EG = dst_pubkey.get_point();
        let C1_EG = src_ciphertext.message_comm.get_point();
        let D1_EG = src_ciphertext.decrypt_handle.get_point();
        let C2_EG = dst_ciphertext.message_comm.get_point();
        let D2_EG = dst_ciphertext.decrypt_handle.get_point();

        // include Y_0, Y_1, Y_2 to transcript and extract challenges
        transcript.validate_and_append_point(b"Y_0", &self.Y_0)?;
        transcript.validate_and_append_point(b"Y_1", &self.Y_1)?;
        transcript.validate_and_append_point(b"Y_2", &self.Y_2)?;

        let c = transcript.challenge_scalar(b"c");
        let w = transcript.challenge_scalar(b"w");
        let ww = w * w;

        // check that the required algebraic condition holds
        let Y_0 = self.Y_0.decompress().ok_or(ProofError::VerificationError)?;
        let Y_1 = self.Y_1.decompress().ok_or(ProofError::VerificationError)?;
        let Y_2 = self.Y_2.decompress().ok_or(ProofError::VerificationError)?;

        let check = RistrettoPoint::vartime_multiscalar_mul(
            vec![
                // that s_1 is the secret key for P1_EG
                self.sh_1,
                -c,
                -Scalar::one(),

                // that r_2 is the randomness used in D2_EG
                w * self.rh_2,
                -w * c,
                -w,

                // that the messages in C1_EG and C2_EG are equal under s_1 and r_2
                ww * c,
                -ww * c,
                ww * self.sh_1,
                -ww * self.rh_2,
                -ww,
            ],
            vec![
                P1_EG, H, Y_0,
                P2_EG, D2_EG, Y_1,
                C2_EG, C1_EG, D1_EG, H, Y_2,
            ],
        );

        if check.is_identity() {
            Ok(())
        } else {
            Err(ProofError::VerificationError)
        }
    }

    pub fn to_bytes(&self) -> [u8; 160] {
        let mut buf = [0_u8; 160];
        buf[..32].copy_from_slice(self.Y_0.as_bytes());
        buf[32..64].copy_from_slice(self.Y_1.as_bytes());
        buf[64..96].copy_from_slice(self.Y_2.as_bytes());
        buf[96..128].copy_from_slice(self.sh_1.as_bytes());
        buf[128..160].copy_from_slice(self.rh_2.as_bytes());
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProofError> {
        let bytes = array_ref![bytes, 0, 160];
        let (Y_0, Y_1, Y_2, sh_1, rh_2) = array_refs![bytes, 32, 32, 32, 32, 32];

        let Y_0 = CompressedRistretto::from_slice(Y_0);
        let Y_1 = CompressedRistretto::from_slice(Y_1);
        let Y_2 = CompressedRistretto::from_slice(Y_2);

        let sh_1 = Scalar::from_canonical_bytes(*sh_1).ok_or(ProofError::FormatError)?;
        let rh_2 = Scalar::from_canonical_bytes(*rh_2).ok_or(ProofError::FormatError)?;

        Ok(EqualityProof {
            Y_0,
            Y_1,
            Y_2,
            sh_1,
            rh_2,
        })
    }
}
