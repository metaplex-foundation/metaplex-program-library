use super::pod;
pub use target_arch::*;

impl From<(pod::PedersenCommitment, pod::PedersenDecryptHandle)> for pod::ElGamalCiphertext {
    fn from((comm, decrypt_handle): (pod::PedersenCommitment, pod::PedersenDecryptHandle)) -> Self {
        let mut buf = [0_u8; 64];
        buf[..32].copy_from_slice(&comm.0);
        buf[32..].copy_from_slice(&decrypt_handle.0);
        pod::ElGamalCiphertext(buf)
    }
}

#[cfg(not(target_arch = "bpf"))]
mod target_arch {
    use {
        super::pod,
        crate::{
            encryption::{
                elgamal::{ElGamalCiphertext, ElGamalPubkey},
                pedersen::{PedersenCommitment, PedersenDecryptHandle},
            },
            errors::ProofError,
        },
        curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar},
        std::convert::TryFrom,
    };

    impl From<Scalar> for pod::Scalar {
        fn from(scalar: Scalar) -> Self {
            Self(scalar.to_bytes())
        }
    }

    impl From<pod::Scalar> for Scalar {
        fn from(pod: pod::Scalar) -> Self {
            Scalar::from_bits(pod.0)
        }
    }

    impl From<ElGamalCiphertext> for pod::ElGamalCiphertext {
        fn from(ct: ElGamalCiphertext) -> Self {
            Self(ct.to_bytes())
        }
    }

    impl TryFrom<pod::ElGamalCiphertext> for ElGamalCiphertext {
        type Error = ProofError;

        fn try_from(ct: pod::ElGamalCiphertext) -> Result<Self, Self::Error> {
            Self::from_bytes(&ct.0).ok_or(ProofError::InconsistentCTData)
        }
    }

    impl From<ElGamalPubkey> for pod::ElGamalPubkey {
        fn from(pk: ElGamalPubkey) -> Self {
            Self(pk.to_bytes())
        }
    }

    impl TryFrom<pod::ElGamalPubkey> for ElGamalPubkey {
        type Error = ProofError;

        fn try_from(pk: pod::ElGamalPubkey) -> Result<Self, Self::Error> {
            Self::from_bytes(&pk.0).ok_or(ProofError::InconsistentCTData)
        }
    }

    impl From<CompressedRistretto> for pod::CompressedRistretto {
        fn from(cr: CompressedRistretto) -> Self {
            Self(cr.to_bytes())
        }
    }

    impl From<pod::CompressedRistretto> for CompressedRistretto {
        fn from(pod: pod::CompressedRistretto) -> Self {
            Self(pod.0)
        }
    }

    impl From<PedersenCommitment> for pod::PedersenCommitment {
        fn from(comm: PedersenCommitment) -> Self {
            Self(comm.to_bytes())
        }
    }

    // For proof verification, interpret pod::PedersenComm directly as CompressedRistretto
    #[cfg(not(target_arch = "bpf"))]
    impl From<pod::PedersenCommitment> for CompressedRistretto {
        fn from(pod: pod::PedersenCommitment) -> Self {
            Self(pod.0)
        }
    }

    #[cfg(not(target_arch = "bpf"))]
    impl TryFrom<pod::PedersenCommitment> for PedersenCommitment {
        type Error = ProofError;

        fn try_from(pod: pod::PedersenCommitment) -> Result<Self, Self::Error> {
            Self::from_bytes(&pod.0).ok_or(ProofError::InconsistentCTData)
        }
    }

    #[cfg(not(target_arch = "bpf"))]
    impl From<PedersenDecryptHandle> for pod::PedersenDecryptHandle {
        fn from(handle: PedersenDecryptHandle) -> Self {
            Self(handle.to_bytes())
        }
    }

    // For proof verification, interpret pod::PedersenDecHandle as CompressedRistretto
    #[cfg(not(target_arch = "bpf"))]
    impl From<pod::PedersenDecryptHandle> for CompressedRistretto {
        fn from(pod: pod::PedersenDecryptHandle) -> Self {
            Self(pod.0)
        }
    }

    #[cfg(not(target_arch = "bpf"))]
    impl TryFrom<pod::PedersenDecryptHandle> for PedersenDecryptHandle {
        type Error = ProofError;

        fn try_from(pod: pod::PedersenDecryptHandle) -> Result<Self, Self::Error> {
            Self::from_bytes(&pod.0).ok_or(ProofError::InconsistentCTData)
        }
    }
}

#[cfg(target_arch = "bpf")]
#[allow(unused_variables)]
mod target_arch {}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{encryption::pedersen::Pedersen, range_proof::RangeProof},
        merlin::Transcript,
        std::convert::TryInto,
    };

    #[test]
    fn test_pod_range_proof_64() {
        let (comm, open) = Pedersen::new(55_u64);

        let mut transcript_create = Transcript::new(b"Test");
        let mut transcript_verify = Transcript::new(b"Test");

        let proof = RangeProof::create(vec![55], vec![64], vec![&open], &mut transcript_create);

        let proof_serialized: pod::RangeProof64 = proof.try_into().unwrap();
        let proof_deserialized: RangeProof = proof_serialized.try_into().unwrap();

        assert!(proof_deserialized
            .verify(
                vec![&comm.get_point().compress()],
                vec![64],
                &mut transcript_verify
            )
            .is_ok());

        // should fail to serialize to pod::RangeProof128
        let proof = RangeProof::create(vec![55], vec![64], vec![&open], &mut transcript_create);

        assert!(TryInto::<pod::RangeProof128>::try_into(proof).is_err());
    }

    #[test]
    fn test_pod_range_proof_128() {
        let (comm_1, open_1) = Pedersen::new(55_u64);
        let (comm_2, open_2) = Pedersen::new(77_u64);
        let (comm_3, open_3) = Pedersen::new(99_u64);

        let mut transcript_create = Transcript::new(b"Test");
        let mut transcript_verify = Transcript::new(b"Test");

        let proof = RangeProof::create(
            vec![55, 77, 99],
            vec![64, 32, 32],
            vec![&open_1, &open_2, &open_3],
            &mut transcript_create,
        );

        let comm_1_point = comm_1.get_point().compress();
        let comm_2_point = comm_2.get_point().compress();
        let comm_3_point = comm_3.get_point().compress();

        let proof_serialized: pod::RangeProof128 = proof.try_into().unwrap();
        let proof_deserialized: RangeProof = proof_serialized.try_into().unwrap();

        assert!(proof_deserialized
            .verify(
                vec![&comm_1_point, &comm_2_point, &comm_3_point],
                vec![64, 32, 32],
                &mut transcript_verify,
            )
            .is_ok());

        // should fail to serialize to pod::RangeProof64
        let proof = RangeProof::create(
            vec![55, 77, 99],
            vec![64, 32, 32],
            vec![&open_1, &open_2, &open_3],
            &mut transcript_create,
        );

        assert!(TryInto::<pod::RangeProof64>::try_into(proof).is_err());
    }
}
