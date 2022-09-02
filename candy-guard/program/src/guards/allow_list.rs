use super::*;
use crate::utils::assert_keys_equal;

/// Configurations options for allow list.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AllowList {
    /// Name of the allowed list.
    pub uuid: [u8; 6],
    /// Merkle root of the addresses allowed to mint. If the root is not
    /// set, any address is allowed to mint.
    pub merkle_root: Option<[u8; 32]>,
    /// Start date for the mint.
    pub start_date: i64,
    /// End date for the mint.
    pub end_date: i64,
    /// Limit of mints per individual address. If this is not set, there
    /// is no limit on the number of mints.
    pub limit: Option<u32>,
    /// Price applicable for the allowed addresses.
    pub price: Option<u64>,
}

/// PDA to track the number of mints for an individual address.
#[account]
#[derive(Default)]
pub struct Allowance {
    pub mint_count: u32,
}

impl Guard for AllowList {
    fn size() -> usize {
        32   // name
        + 33 // option + merkle_root
        +  8 // start_date
        +  8 // end_date
        +  5 // option + u32
    }

    fn mask() -> u64 {
        0b1u64 << 8
    }
}

impl Condition for AllowList {
    fn validate<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        mint_args: &MintArgs,
        _candy_guard_data: &CandyGuardData,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        // (1) checks if the mint time is within the allowed range

        let clock = Clock::get()?;

        if clock.unix_timestamp < self.start_date || clock.unix_timestamp > self.end_date {
            return err!(CandyGuardError::InvalidMintTime);
        }

        // (2) checks that the user is in the allowed list

        let user = ctx.accounts.payer.key();

        if let Some(merkle_root) = &self.merkle_root {
            if let Some(merkle_proof) = &mint_args.merkle_proof {
                if !verify(merkle_proof, merkle_root, &user.to_bytes()) {
                    return err!(CandyGuardError::AddressNotFoundInAllowedList);
                }
            } else {
                return err!(CandyGuardError::MissingAllowedListProof);
            }
        }

        // (3) checks the mint allowance limit

        if self.limit.is_some() {
            let allowance_account =
                Self::get_account_info(ctx, evaluation_context.remaining_account_counter)?;
            evaluation_context.indices.insert(
                "allowlist_index",
                evaluation_context.remaining_account_counter,
            );
            evaluation_context.remaining_account_counter += 1;

            let user = ctx.accounts.payer.key();
            let candy_guard_key = &ctx.accounts.candy_guard.key();

            let seeds = [&self.uuid, user.as_ref(), candy_guard_key.as_ref()];
            let (pda, _) = Pubkey::find_program_address(&seeds, &crate::ID);

            assert_keys_equal(allowance_account.key, &pda)?;

            let account_data = allowance_account.data.borrow();
            let allowance = Allowance::try_from_slice(&account_data)?;

            if let Some(limit) = self.limit {
                if allowance.mint_count >= limit {
                    return err!(CandyGuardError::AllowedMintLimitReached);
                }
            }
        }

        Ok(())
    }

    fn pre_actions<'info>(
        &self,
        ctx: &Context<'_, '_, '_, 'info, Mint<'info>>,
        _mint_args: &MintArgs,
        _candy_guard_data: &CandyGuardData,
        evaluation_context: &mut EvaluationContext,
    ) -> Result<()> {
        if self.limit.is_some() {
            let allowance_account =
                Self::get_account_info(ctx, evaluation_context.indices["allowlist_index"])?;

            let user = ctx.accounts.payer.key();
            let candy_guard_key = &ctx.accounts.candy_guard.key();

            let seeds = [&self.uuid, user.as_ref(), candy_guard_key.as_ref()];
            let (pda, _) = Pubkey::find_program_address(&seeds, &crate::ID);

            assert_keys_equal(allowance_account.key, &pda)?;

            let mut account_data = allowance_account.try_borrow_mut_data()?;
            let mut allowance = Allowance::try_from_slice(&account_data)?;
            allowance.mint_count += 1;
            // saves the changes back to the pda
            let data = &mut allowance.try_to_vec().unwrap();
            account_data[0..data.len()].copy_from_slice(&data);
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
