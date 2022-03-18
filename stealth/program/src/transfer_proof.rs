#[cfg(not(target_arch = "bpf"))]
use {
    crate::encryption::{
        elgamal::{CipherKey, ElGamalKeypair, ElGamalSecretKey},
        pedersen::{Pedersen, PedersenOpening},
    },
};
use {
    borsh::{BorshSerialize, BorshDeserialize},
    bytemuck::{Pod, Zeroable},
    crate::zk_token_elgamal::pod,
    crate::equality_proof::{
        EqualityProof,
        PodEqualityProof,
    },
    crate::{
        errors::ProofError,
        transcript::TranscriptProtocol,
        encryption::elgamal::{
            ElGamalCiphertext,
            ElGamalPubkey,
        },
    },
    merlin::Transcript,
    solana_program::msg,
    std::convert::TryInto,
};

pub trait Verifiable {
    fn verify(&self) -> Result<(), ProofError>;
}

#[cfg(not(target_arch = "bpf"))]
#[derive(Debug, Copy, Clone)]
pub enum Role {
    Source,
    Dest,
}

#[derive(Clone, Copy, Pod, Zeroable, BorshSerialize, BorshDeserialize)]
#[repr(C)]
pub struct TransferData {
    /// The public encryption keys associated with the transfer: source, dest, and auditor
    pub transfer_public_keys: TransferPubkeys, // 64 bytes

    /// The cipher key encrypted by the source pubkey
    pub src_cipher_key_chunk_ct: pod::ElGamalCiphertext, // 64 bytes

    /// The cipher key encrypted by the destination pubkey
    pub dst_cipher_key_chunk_ct: pod::ElGamalCiphertext, // 64 bytes

    /// Zero-knowledge proofs for Transfer
    pub proof: TransferProof,
}

#[cfg(not(target_arch = "bpf"))]
impl TransferData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        src_keypair: &ElGamalKeypair,
        dst_pubkey: ElGamalPubkey,
        cipher_key: CipherKey,
        src_cipher_key_chunk_ct: ElGamalCiphertext,
    ) -> Self {
        let (dst_comm, dst_opening) = Pedersen::new(cipher_key);

        let dst_handle = dst_pubkey.decrypt_handle(&dst_opening);

        let dst_cipher_key_chunk_ct = ElGamalCiphertext {
            message_comm: dst_comm,
            decrypt_handle: dst_handle,
        };

        // grouping of the public keys for the transfer
        let transfer_public_keys = TransferPubkeys {
            src_pubkey: src_keypair.public.into(),
            dst_pubkey: dst_pubkey.into(),
        };

        let proof = TransferProof::new(
            src_keypair,
            &dst_pubkey,
            &src_cipher_key_chunk_ct,
            &dst_cipher_key_chunk_ct,
            &dst_opening,
        );

        Self {
            transfer_public_keys,
            src_cipher_key_chunk_ct: src_cipher_key_chunk_ct.into(),
            dst_cipher_key_chunk_ct: dst_cipher_key_chunk_ct.into(),
            proof,
        }
    }

    /// Extracts the ciphertexts associated with a transfer data
    fn ciphertext(&self, role: Role) -> Result<ElGamalCiphertext, ProofError> {
        match role {
            Role::Source => self.src_cipher_key_chunk_ct,
            Role::Dest => self.dst_cipher_key_chunk_ct,
        }.try_into()
    }

    /// Decrypts transfer amount from transfer data
    pub fn decrypt(&self, role: Role, sk: &ElGamalSecretKey) -> Result<CipherKey, ProofError> {
        let ciphertext = self.ciphertext(role)?;

        sk.decrypt(&ciphertext)
    }
}

impl Verifiable for TransferData {
    fn verify(&self) -> Result<(), ProofError> {
        self.proof.verify(
            &self.src_cipher_key_chunk_ct,
            &self.dst_cipher_key_chunk_ct,
            &self.transfer_public_keys,
        )
    }
}

#[derive(Clone, Copy, Pod, Zeroable, BorshSerialize, BorshDeserialize)]
#[repr(C)]
pub struct TransferProof {
    /// Associated equality proof
    pub equality_proof: PodEqualityProof,
}

// plumbing BS
#[allow(non_snake_case)]
impl TransferProof {
    pub fn transcript_new() -> Transcript {
        Transcript::new(b"TransferProof")
    }

    #[cfg(not(target_arch = "bpf"))]
    pub fn new(
        src_keypair: &ElGamalKeypair,
        dst_pubkey: &ElGamalPubkey,
        src_cipher_key_chunk_ct: &ElGamalCiphertext,
        dst_cipher_key_chunk_ct: &ElGamalCiphertext,
        dst_opening: &PedersenOpening,
    ) -> Self {
        let mut transcript = Self::transcript_new();

        // add a domain separator to record the start of the protocol
        transcript.transfer_proof_domain_sep();

        // extract the relevant scalar and Ristretto points from the inputs
        let P1_EG = src_keypair.public.get_point();
        let C1_EG = src_cipher_key_chunk_ct.message_comm.get_point();
        let D1_EG = src_cipher_key_chunk_ct.decrypt_handle.get_point();

        let P2_EG = dst_pubkey.get_point();
        let C2_EG = dst_cipher_key_chunk_ct.message_comm.get_point();
        let D2_EG = dst_cipher_key_chunk_ct.decrypt_handle.get_point();

        // append all current state to the transcript
        transcript.append_point(b"P1_EG", &P1_EG.compress());
        transcript.append_point(b"C1_EG", &C1_EG.compress());
        transcript.append_point(b"D1_EG", &D1_EG.compress());

        transcript.append_point(b"P2_EG", &P2_EG.compress());
        transcript.append_point(b"C2_EG", &C2_EG.compress());
        transcript.append_point(b"D2_EG", &D2_EG.compress());

        // generate equality_proof
        let equality_proof = EqualityProof::new(
            src_keypair,
            dst_pubkey,
            src_cipher_key_chunk_ct,
            dst_opening,
            &mut transcript,
        );

        Self {
            equality_proof: equality_proof.try_into().expect("equality proof"),
        }
    }

    pub fn build_transcript(
        src_cipher_key_chunk_ct: &pod::ElGamalCiphertext,
        dst_cipher_key_chunk_ct: &pod::ElGamalCiphertext,
        transfer_pubkeys: &TransferPubkeys,
        transcript: &mut Transcript,
    ) -> Result<(), ProofError> {
        // add a domain separator to record the start of the protocol
        transcript.transfer_proof_domain_sep();

        // append all current state to the transcript
        use curve25519_dalek::ristretto::CompressedRistretto;
        transcript.append_point(b"P1_EG", &CompressedRistretto::from_slice(&transfer_pubkeys.src_pubkey.0));
        transcript.append_point(b"C1_EG", &CompressedRistretto::from_slice(&src_cipher_key_chunk_ct.0[..32]));
        transcript.append_point(b"D1_EG", &CompressedRistretto::from_slice(&src_cipher_key_chunk_ct.0[32..]));

        transcript.append_point(b"P2_EG", &CompressedRistretto::from_slice(&transfer_pubkeys.dst_pubkey.0));
        transcript.append_point(b"C2_EG", &CompressedRistretto::from_slice(&dst_cipher_key_chunk_ct.0[..32]));
        transcript.append_point(b"D2_EG", &CompressedRistretto::from_slice(&dst_cipher_key_chunk_ct.0[32..]));

        Ok(())
    }

    pub fn verify(
        self,
        src_cipher_key_chunk_ct: &pod::ElGamalCiphertext,
        dst_cipher_key_chunk_ct: &pod::ElGamalCiphertext,
        transfer_pubkeys: &TransferPubkeys,
    ) -> Result<(), ProofError> {
        let mut transcript = Self::transcript_new();

        TransferProof::build_transcript(
            &src_cipher_key_chunk_ct,
            &dst_cipher_key_chunk_ct,
            &transfer_pubkeys,
            &mut transcript,
        )?;

        let equality_proof: EqualityProof = self.equality_proof.try_into()?;

        solana_program::log::sol_log_compute_units();

        // extract the relevant scalar and Ristretto points from the inputs
        msg!("Extracting points from inputs");
        let src_pubkey: ElGamalPubkey = transfer_pubkeys.src_pubkey.try_into()?;
        let dst_pubkey: ElGamalPubkey = transfer_pubkeys.dst_pubkey.try_into()?;

        msg!("Extracting cipher text from inputs");
        let src_cipher_key_chunk_ct: ElGamalCiphertext = (*src_cipher_key_chunk_ct).try_into()?;
        let dst_cipher_key_chunk_ct: ElGamalCiphertext = (*dst_cipher_key_chunk_ct).try_into()?;

        // verify equality proof
        msg!("Verifying equality proof");
        equality_proof.verify(
            &src_pubkey,
            &dst_pubkey,
            &src_cipher_key_chunk_ct,
            &dst_cipher_key_chunk_ct,
            &mut transcript
        )?;

        Ok(())
    }
}

/// The ElGamal public keys needed for a transfer
#[derive(Clone, Copy, Pod, Zeroable, BorshSerialize, BorshDeserialize)]
#[repr(C)]
pub struct TransferPubkeys {
    pub src_pubkey: pod::ElGamalPubkey,     // 32 bytes
    pub dst_pubkey: pod::ElGamalPubkey,     // 32 bytes
}

#[cfg(test)]
mod test {
    use super::*;
    use curve25519_dalek::scalar::Scalar;

    #[test]
    fn test_transfer_decryption() {
        // ElGamalKeypair keys for source, destination, and auditor accounts
        let src_keypair = ElGamalKeypair::default();
        let dst_keypair = ElGamalKeypair::default();

        let cipher_key_chunk: u32 = 77;
        let cipher_key = CipherKey(Scalar::from(cipher_key_chunk).bytes[..24].try_into().unwrap());
        let cipher_key_ct = src_keypair.public.encrypt(cipher_key);

        // create transfer data
        let transfer_data = TransferData::new(
            &src_keypair,
            dst_keypair.public,
            cipher_key,
            cipher_key_ct,
        );

        assert_eq!(
            transfer_data.decrypt(Role::Source, &src_keypair.secret),
            Ok(cipher_key),
        );

        assert_eq!(
            transfer_data.decrypt(Role::Dest, &dst_keypair.secret),
            Ok(cipher_key),
        );
    }

    #[test]
    fn test_transfer_correctness() {
        // ElGamalKeypair keys for source, destination, and auditor accounts
        let src_keypair = ElGamalKeypair::default();
        let dst_pubkey = ElGamalKeypair::default().public;

        let cipher_key_chunk: u32 = 77;
        let cipher_key = CipherKey(Scalar::from(cipher_key_chunk).bytes[..24].try_into().unwrap());
        let cipher_key_ct = src_keypair.public.encrypt(cipher_key);

        // create transfer data
        let transfer_data = TransferData::new(
            &src_keypair,
            dst_pubkey,
            cipher_key,
            cipher_key_ct,
        );

        assert_eq!(transfer_data.verify(), Ok(()));
    }

    #[test]
    fn test_transfer_failure() {
        // ElGamalKeypair keys for source, destination, and auditor accounts
        let src_keypair = ElGamalKeypair::default();
        let dst_pubkey = ElGamalKeypair::default().public;

        let cipher_key_chunk: u32 = 77;
        let cipher_key = CipherKey(Scalar::from(cipher_key_chunk).bytes[..24].try_into().unwrap());
        let cipher_key_ct = src_keypair.public.encrypt(cipher_key);

        let wrong_cipher_key = CipherKey(Scalar::from(cipher_key_chunk + 1).bytes[..24].try_into().unwrap());

        // create transfer data
        let transfer_data = TransferData::new(
            &src_keypair,
            dst_pubkey,
            wrong_cipher_key,
            cipher_key_ct,
        );

        assert!(transfer_data.verify().is_err());
    }
}
