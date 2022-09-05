use super::*;

/// Configurations options for allow list.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AllowList {
    /// Merkle root of the addresses allowed to mint.
    pub merkle_root: [u8; 32],
}

impl Guard for AllowList {
    fn size() -> usize {
        32 // merkle_root
    }

    fn mask() -> u64 {
        0b1u64 << 8
    }
}

impl Condition for AllowList {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        mint_args: &[u8],
        _guard_set: &GuardSet,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        let user = ctx.accounts.payer.key();

        let merkle_proof: Vec<[u8; 32]> =
            if let Ok(proof) = Vec::try_from_slice(&mint_args[evaluation_context.args_cursor..]) {
                proof
            } else {
                return err!(CandyGuardError::MissingAllowedListProof);
            };
        // updates the number of bytes read
        evaluation_context.args_cursor += 4 + (merkle_proof.len() * 32);

        if !verify(&merkle_proof[..], &self.merkle_root, &user.to_bytes()) {
            return err!(CandyGuardError::AddressNotFoundInAllowedList);
        }

        Ok(())
    }
}

// These functions deal with verification of Merkle trees (hash trees).
// Direct port of:
// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v3.4.0/contracts/cryptography/MerkleProof.sol

/// Returns true if a `leaf` can be proved to be a part of a Merkle tree
/// defined by `root`. For this, a `proof` must be provided, containing
/// sibling hashes on the branch from the leaf to the root of the tree. Each
/// pair of leaves and each pair of pre-images are assumed to be sorted.
fn verify(proof: &[[u8; 32]], root: &[u8; 32], leaf: &[u8; 32]) -> bool {
    let mut computed_hash = *leaf;
    for proof_element in proof.iter() {
        if computed_hash <= *proof_element {
            // Hash(current computed hash + current element of the proof)
            computed_hash =
                solana_program::keccak::hashv(&[&[0x01], &computed_hash, proof_element]).0
        } else {
            // Hash(current element of the proof + current computed hash)
            computed_hash =
                solana_program::keccak::hashv(&[&[0x01], proof_element, &computed_hash]).0;
        }
    }
    // Check if the computed hash (root) is equal to the provided root
    computed_hash == *root
}
