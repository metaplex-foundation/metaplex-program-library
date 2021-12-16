use {
    spl_zk_token_sdk::zk_token_elgamal::pod,
    bytemuck::{Pod, Zeroable},
};
#[cfg(not(target_arch = "bpf"))]
use {
    crate::equality_proof::{
        EqualityProof,
        PodEqualityProof,
    },
    spl_zk_token_sdk::{
        errors::ProofError,
        transcript::TranscriptProtocol,
        encryption::{
            discrete_log::*,
            elgamal::{ElGamalCiphertext, ElGamalKeypair, ElGamalPubkey, ElGamalSecretKey},
            pedersen::{Pedersen, PedersenCommitment, PedersenDecryptHandle, PedersenOpening},
        },
    },
    curve25519_dalek::scalar::Scalar,
    merlin::Transcript,
    std::convert::TryInto,
};

#[cfg(not(target_arch = "bpf"))]
pub trait Verifiable {
    fn verify(&self) -> Result<(), ProofError>;
}

#[cfg(not(target_arch = "bpf"))]
#[derive(Debug, Copy, Clone)]
pub enum Role {
    Source,
    Dest,
}

// TODO: remove wrapper?
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct TransferData {
    /// The public encryption keys associated with the transfer: source, dest, and auditor
    /// TODO: auditor
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
        cipher_key_chunk: u32,
        src_cipher_key_chunk_ct: ElGamalCiphertext,
    ) -> Self {
        let (dst_comm, dst_opening) = Pedersen::new(cipher_key_chunk);

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
    ///
    /// TODO: This function should run in constant time. Use `subtle::Choice` for the if statement
    /// and make sure that the function does not terminate prematurely due to errors
    ///
    /// TODO: Define specific error type for decryption error
    pub fn decrypt_amount(&self, role: Role, sk: &ElGamalSecretKey) -> Result<u32, ProofError> {
        let ciphertext = self.ciphertext(role)?;

        let key_chunk = ciphertext.decrypt_u32_online(sk, &DECODE_U32_PRECOMPUTATION_FOR_G);

        if let Some(key_chunk) = key_chunk {
            Ok(key_chunk)
        } else {
            Err(ProofError::VerificationError)
        }
    }
}

#[cfg(not(target_arch = "bpf"))]
impl Verifiable for TransferData {
    fn verify(&self) -> Result<(), ProofError> {
        self.proof.verify(
            &self.src_cipher_key_chunk_ct,
            &self.dst_cipher_key_chunk_ct,
            &self.transfer_public_keys,
        )
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct TransferProof {
    /// Associated equality proof
    pub equality_proof: PodEqualityProof,
}

// plumbing BS
#[allow(non_snake_case)]
#[cfg(not(target_arch = "bpf"))]
impl TransferProof {
    fn transcript_new() -> Transcript {
        Transcript::new(b"TransferProof")
    }

    pub fn new(
        src_keypair: &ElGamalKeypair,
        dst_pubkey: &ElGamalPubkey,
        src_cipher_key_chunk_ct: &ElGamalCiphertext,
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

        // append all current state to the transcript
        transcript.append_point(b"P1_EG", &P1_EG.compress());
        transcript.append_point(b"C1_EG", &C1_EG.compress());
        transcript.append_point(b"D1_EG", &D1_EG.compress());

        transcript.append_point(b"P2_EG", &P2_EG.compress());

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

    pub fn verify(
        self,
        src_cipher_key_chunk_ct: &pod::ElGamalCiphertext,
        dst_cipher_key_chunk_ct: &pod::ElGamalCiphertext,
        transfer_pubkeys: &TransferPubkeys,
    ) -> Result<(), ProofError> {
        let mut transcript = Self::transcript_new();

        let equality_proof: EqualityProof = self.equality_proof.try_into()?;

        // add a domain separator to record the start of the protocol
        transcript.transfer_proof_domain_sep();

        // extract the relevant scalar and Ristretto points from the inputs
        let src_pubkey: ElGamalPubkey = transfer_pubkeys.src_pubkey.try_into()?;
        let dst_pubkey: ElGamalPubkey = transfer_pubkeys.dst_pubkey.try_into()?;

        let src_cipher_key_chunk_ct: ElGamalCiphertext = (*src_cipher_key_chunk_ct).try_into()?;
        let dst_cipher_key_chunk_ct: ElGamalCiphertext = (*dst_cipher_key_chunk_ct).try_into()?;

        let P1_EG = src_pubkey.get_point();
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

        // verify equality proof
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
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct TransferPubkeys {
    pub src_pubkey: pod::ElGamalPubkey,     // 32 bytes
    pub dst_pubkey: pod::ElGamalPubkey,     // 32 bytes
}
