//! Program for distributing tokens efficiently via uploading a Merkle root.
use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program::{invoke, invoke_signed},
        system_instruction, sysvar,
    },
};
use anchor_spl::token::{self, Token, TokenAccount};
use mpl_token_metadata;
use std::io::Write;

pub mod merkle_proof;

declare_id!("gdrpGjVffourzkdDRrQmySw4aTHr8a3xmQzzxSwFD1a");
pub const CANDY_MACHINE_V1_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0x09, 0x2a, 0xee, 0x40, 0xbb, 0xdd, 0x63, 0x1e, 0xef, 0xfb, 0x7c, 0x96, 0xf6, 0x15, 0x65, 0x76,
    0x84, 0x65, 0xf3, 0xc1, 0x9c, 0xf9, 0x90, 0xcd, 0x7f, 0x74, 0x8c, 0x8d, 0x79, 0x95, 0x08, 0x20,
]);

pub const CANDY_MACHINE_V2_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0x09, 0x2a, 0xee, 0x3d, 0xfc, 0x2d, 0x0e, 0x55, 0x78, 0x23, 0x13, 0x83, 0x79, 0x69, 0xea, 0xf5,
    0x21, 0x51, 0xc0, 0x96, 0xc0, 0x6b, 0x5c, 0x2a, 0x82, 0xf0, 0x86, 0xa5, 0x03, 0xe8, 0x2c, 0x34,
]);


fn verify_candy(candy_machine_program_account: &Pubkey) -> Result<()> {
    if candy_machine_program_account != &CANDY_MACHINE_V1_PROGRAM_ID
        && candy_machine_program_account != &CANDY_MACHINE_V2_PROGRAM_ID
    {
        return Err(GumdropError::MustUseOfficialCandyMachine.into());
    }
    Ok(())
}

const CLAIM_COUNT: &[u8] = b"ClaimCount";
const CLAIM_STATUS: &[u8] = b"ClaimStatus";

fn verify_temporal<'a>(
    distributor: &Account<'a, MerkleDistributor>,
    temporal: &Signer<'a>,
    claimant_secret: Pubkey,
) -> Result<()> {
    require!(
        // got the OTP auth from the signer specified by the creator
        temporal.key() == distributor.temporal
        // the secret used in the hash was a Pubkey (wallet) so proof-of-ownership is achieved by
        // signing for this transaction
        || temporal.key() == claimant_secret
        // the creator decided not to use a temporal signer
        || distributor.temporal == Pubkey::default()
        ,
        GumdropError::TemporalMismatch
    );

    Ok(())
}

fn verify_claim_bump<'a>(
    claim_account: &AccountInfo<'a>,
    claim_prefix: &[u8],
    claim_bump: u8,
    index: u64,
    distributor: &Account<'a, MerkleDistributor>,
) -> Result<()> {
    require!(
        claim_prefix == CLAIM_COUNT
        || claim_prefix == CLAIM_STATUS,
        GumdropError::InvalidClaimBump,
    );

    let (claim_account_key, claim_account_bump) = Pubkey::find_program_address(
        &[
            claim_prefix,
            &index.to_le_bytes(),
            &distributor.key().to_bytes(),
        ],
        &ID,
    );
    require!(
        claim_account_key == *claim_account.key
        && claim_account_bump == claim_bump,
        GumdropError::InvalidClaimBump,
    );

    Ok(())
}

fn get_or_create_claim_count<'a>(
    distributor: &Account<'a, MerkleDistributor>,
    claim_count: &AccountInfo<'a>,
    temporal: &Signer<'a>,
    payer: &Signer<'a>,
    system_program: &Program<'a, System>,
    claim_bump: u8,
    index: u64,
    claimant_secret: Pubkey,
) -> Result<Account<'a, ClaimCount>> {
    let rent = &Rent::get()?;
    let space = 8 + ClaimCount::default().try_to_vec().unwrap().len();

    verify_claim_bump(claim_count, CLAIM_COUNT, claim_bump, index, distributor)?;

    let create_claim_state = claim_count.lamports() == 0; // TODO: support initial lamports?
    if create_claim_state {
        let lamports = rent.minimum_balance(space);
        let claim_count_seeds = [
            CLAIM_COUNT.as_ref(),
            &index.to_le_bytes(),
            &distributor.key().to_bytes(),
            &[claim_bump],
        ];

        invoke_signed(
            &system_instruction::create_account(
                &payer.key(),
                claim_count.key,
                lamports,
                space as u64,
                &ID),
            &[
                payer.to_account_info().clone(),
                claim_count.clone(),
                system_program.to_account_info().clone(),
            ],
            &[&claim_count_seeds],
        )?;

        let mut data = claim_count.try_borrow_mut_data()?;
        let dst: &mut [u8] = &mut data;
        let mut cursor = std::io::Cursor::new(dst);
        cursor.write_all(&<ClaimCount as anchor_lang::Discriminator>::discriminator()).unwrap();
    }

    // anchor_lang::Account::try_from(&claim_count)?;
    let mut pa: Account<ClaimCount> =
        Account::try_from(&claim_count)?;

    if create_claim_state {
        verify_temporal(distributor, temporal, claimant_secret)?;
        pa.claimant = payer.key();
    } else {
        require!(pa.claimant == payer.key(), GumdropError::OwnerMismatch);
    }

    Ok(pa)
}

/// The [gumdrop] program.
#[program]
pub mod gumdrop {
    use super::*;

    /// Creates a new [MerkleDistributor].
    /// After creating this [MerkleDistributor], the account should be seeded with tokens via
    /// delegates
    pub fn new_distributor(
        ctx: Context<NewDistributor>,
        bump: u8,
        root: [u8; 32],
        temporal: Pubkey,
    ) -> Result<()> {
        let distributor = &mut ctx.accounts.distributor;

        distributor.base = ctx.accounts.base.key();
        distributor.bump = bump;

        distributor.root = root;
        distributor.temporal = temporal;

        Ok(())
    }

    /// Closes distributor-owned token accounts. Normal tokens should just use a delegate but we
    /// need to transfer ownership for edition minting ATM.
    pub fn close_distributor_token_account(
        ctx: Context<CloseDistributorTokenAccount>,
        _bump: u8,
    ) -> Result<()> {
        let distributor = &ctx.accounts.distributor;

        // should be implicit in the PDA
        require!(distributor.base == ctx.accounts.base.key(),  GumdropError::Unauthorized);

        let seeds = [
            b"MerkleDistributor".as_ref(),
            &distributor.base.to_bytes(),
            &[ctx.accounts.distributor.bump],
        ];

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.to.to_account_info(),
                    authority: ctx.accounts.distributor.to_account_info(),
                },
            )
                .with_signer(&[&seeds[..]]),
            ctx.accounts.from.amount,
        )?;

        token::close_account(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::CloseAccount {
                    account: ctx.accounts.from.to_account_info(),
                    destination: ctx.accounts.receiver.to_account_info(),
                    authority: ctx.accounts.distributor.to_account_info(),
                },
            )
                .with_signer(&[&seeds[..]]),
        )?;

        Ok(())
    }

    /// Closes an existing [MerkleDistributor].
    /// Moves all tokens from the [MerkleDistributor] to the specified account and closes
    /// distributor accounts.
    /// Must `close_distributor_token_account` first
    pub fn close_distributor<'info>(
        ctx: Context<'_, '_, '_, 'info, CloseDistributor<'info>>,
        _bump: u8,
        _wallet_bump: u8,
    ) -> Result<()> {
        let distributor = &ctx.accounts.distributor;

        // should be implicit in the PDA
        require!(distributor.base == ctx.accounts.base.key(),  GumdropError::Unauthorized);

        let wallet_seeds = [
            b"Wallet".as_ref(),
            &distributor.key().to_bytes(),
            &[_wallet_bump],
        ];

        if !ctx.remaining_accounts.is_empty() {
            // transfer authority out
            let candy_machine_info = &ctx.remaining_accounts[0];
            let candy_machine_program_info = &ctx.remaining_accounts[1];
            verify_candy(candy_machine_program_info.key)?;
            // TODO. global::update_authority instruction...
            let mut data = vec![
                0x20, 0x2e, 0x40, 0x1c, 0x95, 0x4b, 0xf3, 0x58,
            ];

            data.push(0x01);
            data.extend_from_slice(&ctx.accounts.receiver.key.to_bytes());

            invoke_signed(
                &Instruction {
                    program_id: *candy_machine_program_info.key,
                    accounts: vec![
                        AccountMeta::new(*candy_machine_info.key, false),
                        AccountMeta::new(*ctx.accounts.distributor_wallet.key, true),
                    ],
                    data: data,
                },
                &[
                    candy_machine_info.clone(),
                    ctx.accounts.distributor_wallet.clone(),
                ],
                &[&wallet_seeds],
            )?;
        }

        invoke_signed(
            &system_instruction::transfer(
                ctx.accounts.distributor_wallet.key,
                ctx.accounts.receiver.key,
                ctx.accounts.distributor_wallet.lamports(),
            ),
            &[
                ctx.accounts.distributor_wallet.clone(),
                ctx.accounts.receiver.clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
            &[&wallet_seeds],
        )?;

        Ok(())
    }

    pub fn prove_claim<'info>(
        ctx: Context<ProveClaim>,
        claim_prefix: Vec<u8>,
        claim_bump: u8,
        index: u64,
        amount: u64,
        claimant_secret: Pubkey,
        resource: Pubkey,
        resource_nonce: Vec<u8>,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        // The logic here is that we will allow the proof to be whichever prefix matches the claim
        // type. The ClaimProof will live at the same place as V1 ClaimCount and V1 ClaimStatus so
        // that users can't claim with both endpoints but also maintain some backwards
        // compatibility. The account is created wherever this prefix points to and since the
        // resource is unique per gumdrop, if this is messed up, they shouldn't be able to claim
        // extra resources.
        require!(
            claim_prefix.as_slice() == CLAIM_COUNT
            || claim_prefix.as_slice() == CLAIM_STATUS,
            GumdropError::InvalidProof,
        );

        let claim_proof = &mut ctx.accounts.claim_proof;
        let distributor = &ctx.accounts.distributor;

        verify_claim_bump(
            &claim_proof.to_account_info(),
            claim_prefix.as_slice(),
            claim_bump,
            index,
            distributor,
        )?;

        // Verify the merkle proof.
        let node = if resource_nonce.is_empty() {
            solana_program::keccak::hashv(&[
                &[0x00],
                &index.to_le_bytes(),
                &claimant_secret.to_bytes(),
                &resource.to_bytes(),
                &amount.to_le_bytes(),
            ])
        } else {
            solana_program::keccak::hashv(&[
                &[0x00],
                &index.to_le_bytes(),
                &claimant_secret.to_bytes(),
                &resource.to_bytes(),
                &amount.to_le_bytes(),
                resource_nonce.as_slice(),
            ])
        };
        require!(
            merkle_proof::verify(proof, distributor.root, node.0),
            GumdropError::InvalidProof,
        );

        verify_temporal(distributor, &ctx.accounts.temporal, claimant_secret)?;

        claim_proof.amount = amount;
        claim_proof.count = 0;
        claim_proof.claimant = ctx.accounts.payer.key();
        claim_proof.resource = resource;
        claim_proof.resource_nonce = resource_nonce;

        Ok(())
    }

    /// Claims tokens from the [MerkleDistributor].
    pub fn claim(
        ctx: Context<Claim>,
        claim_bump: u8,
        index: u64,
        amount: u64,
        claimant_secret: Pubkey,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        let claim_status = &mut ctx.accounts.claim_status;
        require!(
            *claim_status.to_account_info().owner == ID,
            GumdropError::OwnerMismatch
        );
        require!(
            // This check is redudant, we should not be able to initialize a claim status account at the same key.
            !claim_status.is_claimed && claim_status.claimed_at == 0,
             GumdropError::DropAlreadyClaimed
        );

        let distributor = &ctx.accounts.distributor;
        let mint = ctx.accounts.from.mint;

        verify_claim_bump(
            &claim_status.to_account_info(),
            CLAIM_STATUS,
            claim_bump,
            index,
            distributor,
        )?;

        // Verify the merkle proof.
        let node = solana_program::keccak::hashv(&[
            &[0x00],
            &index.to_le_bytes(),
            &claimant_secret.to_bytes(),
            &mint.to_bytes(),
            &amount.to_le_bytes(),
        ]);
        require!(
            merkle_proof::verify(proof, distributor.root, node.0),
             GumdropError::InvalidProof
        );

        // Mark it claimed and send the tokens.
        claim_status.amount = amount;
        claim_status.is_claimed = true;
        let clock = Clock::get()?;
        claim_status.claimed_at = clock.unix_timestamp;
        claim_status.claimant = ctx.accounts.payer.key();

        let seeds = [
            b"MerkleDistributor".as_ref(),
            &distributor.base.to_bytes(),
            &[ctx.accounts.distributor.bump],
        ];

        verify_temporal(distributor, &ctx.accounts.temporal, claimant_secret)?;
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.to.to_account_info(),
                    authority: ctx.accounts.distributor.to_account_info(),
                },
            )
                .with_signer(&[&seeds[..]]),
            amount,
        )?;

        emit!(ClaimedEvent {
            index,
            claimant: ctx.accounts.payer.key(),
            amount
        });
        Ok(())
    }

    /// Claims NFTs directly from the candy machine through the [MerkleDistributor].
    pub fn claim_candy<'info>(
        ctx: Context<'_, '_, '_, 'info, ClaimCandy<'info>>,
        wallet_bump: u8,
        claim_bump: u8,
        index: u64,
        amount: u64,
        claimant_secret: Pubkey,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        let distributor = &ctx.accounts.distributor;
        let mut claim_count = get_or_create_claim_count(
            &ctx.accounts.distributor,
            &ctx.accounts.claim_count,
            &ctx.accounts.temporal,
            &ctx.accounts.payer,
            &ctx.accounts.system_program,
            claim_bump,
            index,
            claimant_secret,
        )?;
        require!(
            *claim_count.to_account_info().owner == ID,
             GumdropError::OwnerMismatch
        );

        // TODO: this is a bit weird but we verify elsewhere that the candy_machine_config is
        // actually a config thing and not a mint
        // Verify the merkle proof.
        let node = solana_program::keccak::hashv(&[
            &[0x00],
            &index.to_le_bytes(),
            &claimant_secret.to_bytes(),
            &ctx.accounts.candy_machine_config.key.to_bytes(),
            &amount.to_le_bytes(),
        ]);
        require!(
            merkle_proof::verify(proof, distributor.root, node.0),
             GumdropError::InvalidProof
        );

        // This user is whitelisted to mint at most `amount` NFTs from the candy machine
        require!(
            claim_count.count < amount,
             GumdropError::DropAlreadyClaimed
        );

        // Mark it claimed
        claim_count.count = claim_count.count
            .checked_add(1)
            .ok_or(GumdropError::NumericalOverflow)?;


        issue_mint_nft(
            &distributor,
            &ctx.accounts.distributor_wallet,
            &ctx.accounts.payer,
            &ctx.accounts.candy_machine_config,
            &ctx.accounts.candy_machine,
            &ctx.accounts.candy_machine_wallet,
            &ctx.accounts.candy_machine_mint,
            &ctx.accounts.candy_machine_metadata,
            &ctx.accounts.candy_machine_master_edition,
            &ctx.accounts.system_program,
            &ctx.accounts.token_program,
            &ctx.accounts.token_metadata_program,
            &ctx.accounts.candy_machine_program,
            &ctx.accounts.rent,
            &ctx.accounts.clock,
            &ctx.remaining_accounts,
            wallet_bump,
        )?;

        // reserialize claim_count
        {
            let mut claim_count_data: &mut [u8] = &mut ctx.accounts.claim_count.try_borrow_mut_data()?;
            claim_count.try_serialize(&mut claim_count_data)?;
        }

        Ok(())
    }

    /// Claims NFTs by calling MintNewEditionFromMasterEditionViaToken
    pub fn claim_edition(
        ctx: Context<ClaimEdition>,
        claim_bump: u8,
        index: u64,
        amount: u64,
        edition: u64,
        claimant_secret: Pubkey,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        let distributor = &ctx.accounts.distributor;
        let mut claim_count = get_or_create_claim_count(
            &ctx.accounts.distributor,
            &ctx.accounts.claim_count,
            &ctx.accounts.temporal,
            &ctx.accounts.payer,
            &ctx.accounts.system_program,
            claim_bump,
            index,
            claimant_secret,
        )?;
        require!(
            *claim_count.to_account_info().owner == ID,
             GumdropError::OwnerMismatch
        );

        // TODO: master_edition or something else? should we has the edition here also?
        let node = solana_program::keccak::hashv(&[
            &[0x00],
            &index.to_le_bytes(),
            &claimant_secret.to_bytes(),
            &ctx.accounts.metadata_master_mint.key.to_bytes(),
            &amount.to_le_bytes(),
            &edition.to_le_bytes(),
        ]);
        require!(
            merkle_proof::verify(proof, distributor.root, node.0),
             GumdropError::InvalidProof
        );

        // This user is whitelisted to mint at most `amount` NFTs from the candy machine
        require!(
            claim_count.count < amount,
            GumdropError::DropAlreadyClaimed
        );

        // Mark it claimed
        claim_count.count = claim_count.count
            .checked_add(1)
            .ok_or(GumdropError::NumericalOverflow)?;

        let seeds = [
            b"MerkleDistributor".as_ref(),
            &distributor.base.to_bytes(),
            &[ctx.accounts.distributor.bump],
        ];

        let metadata_infos = [
            ctx.accounts.token_metadata_program.clone(),
            ctx.accounts.metadata_new_metadata.clone(),
            ctx.accounts.metadata_new_edition.clone(),
            ctx.accounts.metadata_master_edition.clone(),
            ctx.accounts.metadata_new_mint.clone(),
            ctx.accounts.metadata_edition_mark_pda.clone(),
            ctx.accounts.metadata_new_mint_authority.to_account_info().clone(),
            ctx.accounts.payer.to_account_info().clone(),
            ctx.accounts.distributor.to_account_info().clone(),
            ctx.accounts.metadata_master_token_account.clone(),
            ctx.accounts.metadata_new_update_authority.clone(),
            ctx.accounts.metadata_master_metadata.clone(),
            ctx.accounts.metadata_master_mint.clone(),
            ctx.accounts.rent.to_account_info().clone(),
        ];

        invoke_signed(
            &mpl_token_metadata::instruction::mint_new_edition_from_master_edition_via_token(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.metadata_new_metadata.key,
                *ctx.accounts.metadata_new_edition.key,
                *ctx.accounts.metadata_master_edition.key,
                *ctx.accounts.metadata_new_mint.key,
                *ctx.accounts.metadata_new_mint_authority.key,
                *ctx.accounts.payer.key,
                ctx.accounts.distributor.key(),
                *ctx.accounts.metadata_master_token_account.key,
                *ctx.accounts.metadata_new_update_authority.key,
                *ctx.accounts.metadata_master_metadata.key,
                *ctx.accounts.metadata_master_mint.key,
                edition,
            ),
            &metadata_infos,
            &[&seeds],
        )?;

        // reserialize claim_count
        {
            let mut claim_count_data: &mut [u8] = &mut ctx.accounts.claim_count.try_borrow_mut_data()?;
            claim_count.try_serialize(&mut claim_count_data)?;
        }

        Ok(())
    }


    pub fn claim_candy_proven<'info>(
        ctx: Context<'_, '_, '_, 'info, ClaimCandyProven<'info>>,
        wallet_bump: u8,
        _claim_bump: u8,   // proof is not created
        _index: u64,
    ) -> Result<()> {
        let claim_proof = &mut ctx.accounts.claim_proof;
        let distributor = &ctx.accounts.distributor;

        require!(
            claim_proof.claimant == ctx.accounts.payer.key(),
            GumdropError::InvalidProof,
        );

        require!(
            claim_proof.resource == *ctx.accounts.candy_machine_config.key,
            GumdropError::InvalidProof,
        );

        // At least 1 remaining
        require!(
            claim_proof.count < claim_proof.amount,
            GumdropError::DropAlreadyClaimed,
        );

        // Mark it claimed
        claim_proof.count = claim_proof.count
            .checked_add(1)
            .ok_or(GumdropError::NumericalOverflow)?;

        issue_mint_nft(
            &distributor,
            &ctx.accounts.distributor_wallet,
            &ctx.accounts.payer,
            &ctx.accounts.candy_machine_config,
            &ctx.accounts.candy_machine,
            &ctx.accounts.candy_machine_wallet,
            &ctx.accounts.candy_machine_mint,
            &ctx.accounts.candy_machine_metadata,
            &ctx.accounts.candy_machine_master_edition,
            &ctx.accounts.system_program,
            &ctx.accounts.token_program,
            &ctx.accounts.token_metadata_program,
            &ctx.accounts.candy_machine_program,
            &ctx.accounts.rent,
            &ctx.accounts.clock,
            &ctx.remaining_accounts,
            wallet_bump,
        )?;

        Ok(())
    }

    pub fn recover_update_authority(
        ctx: Context<RecoverUpdateAuthority>,
        _bump: u8,
        wallet_bump: u8,
    ) -> Result<()> {
        let wallet_seeds = [
            b"Wallet".as_ref(),
            &ctx.accounts.distributor.key().to_bytes(),
            &[wallet_bump],
        ];

        invoke_signed(
            &mpl_token_metadata::instruction::update_metadata_accounts(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.metadata.key,
                *ctx.accounts.distributor_wallet.key,
                Some(*ctx.accounts.new_update_authority.key),
                None,
                None,
            ),
            &[
                ctx.accounts.token_metadata_program.to_account_info(),
                ctx.accounts.metadata.to_account_info(),
                ctx.accounts.distributor_wallet.to_account_info(),
            ],
            &[&wallet_seeds],
        )?;

        Ok(())
    }
}

fn issue_mint_nft<'info>(
    distributor: &Account<'info, MerkleDistributor>,
    distributor_wallet: &AccountInfo<'info>,
    payer: &Signer<'info>,
    candy_machine_config: &AccountInfo<'info>,
    candy_machine: &AccountInfo<'info>,
    candy_machine_wallet: &AccountInfo<'info>,
    candy_machine_mint: &AccountInfo<'info>,
    candy_machine_metadata: &AccountInfo<'info>,
    candy_machine_master_edition: &AccountInfo<'info>,
    system_program: &Program<'info, System>,
    token_program: &Program<'info, Token>,
    token_metadata_program: &AccountInfo<'info>,
    candy_machine_program: &AccountInfo<'info>,
    rent: &Sysvar<'info, Rent>,
    clock: &Sysvar<'info, Clock>,
    claim_remaining_accounts: &[AccountInfo<'info>],
    wallet_bump: u8,
) -> Result<()> {
    // Transfer the required SOL from the payer
    let required_lamports;
    let remaining_accounts;
    {
        let rent = &Rent::get()?;
        let mut candy_machine_data: &[u8] = &candy_machine.try_borrow_data()?;
        verify_candy(candy_machine_program.key)?;
        let candy_machine = CandyMachine::try_deserialize(&mut candy_machine_data)?;
        let required_rent =
            rent.minimum_balance(mpl_token_metadata::state::MAX_METADATA_LEN)
                + rent.minimum_balance(mpl_token_metadata::state::MAX_MASTER_EDITION_LEN);

        if candy_machine.token_mint.is_some() {
            required_lamports = required_rent;

            // checked by candy machine
            let token_account_info = &claim_remaining_accounts[0];
            let transfer_authority_info = &claim_remaining_accounts[1];
            remaining_accounts = vec![
                token_account_info.clone(),
                transfer_authority_info.clone(),
            ];
        } else {
            required_lamports = candy_machine.data.price + required_rent;
            remaining_accounts = vec![];
        }
    }
    msg!(
        "Transferring {} lamports to distributor wallet for candy machine mint",
        required_lamports,
    );
    invoke(
        &system_instruction::transfer(
            payer.key,
            distributor_wallet.key,
            required_lamports,
        ),
        &[
            payer.to_account_info().clone(),
            distributor_wallet.clone(),
            system_program.to_account_info().clone(),
        ],
    )?;

    let wallet_seeds = [
        b"Wallet".as_ref(),
        &distributor.key().to_bytes(),
        &[wallet_bump],
    ];
    let mut account_metas = vec![
        AccountMeta::new_readonly(candy_machine_config.key(), false),
        AccountMeta::new(candy_machine.key(), false),
        AccountMeta::new(distributor_wallet.key(), true),
        AccountMeta::new(candy_machine_wallet.key(), false),
        AccountMeta::new(candy_machine_metadata.key(), false),
        AccountMeta::new(candy_machine_mint.key(), false),
        AccountMeta::new_readonly(payer.key(), true),
        AccountMeta::new_readonly(payer.key(), true),
        AccountMeta::new(candy_machine_master_edition.key(), false),
        AccountMeta::new_readonly(token_metadata_program.key(), false),
        AccountMeta::new_readonly(token_program.key(), false),
        AccountMeta::new_readonly(system_program.key(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
    ];
    for a in &remaining_accounts {
        account_metas.push(AccountMeta::new(a.key(), false));
    }
    let mut candy_machine_infos = vec![
        candy_machine_config.clone(),
        candy_machine.to_account_info().clone(),
        distributor_wallet.clone(),
        candy_machine_wallet.clone(),
        candy_machine_metadata.clone(),
        candy_machine_mint.clone(),
        payer.to_account_info().clone(),
        candy_machine_master_edition.clone(),
        token_metadata_program.clone(),
        token_program.to_account_info().clone(),
        system_program.to_account_info().clone(),
        rent.to_account_info().clone(),
        clock.to_account_info().clone(),
    ];
    candy_machine_infos.extend(remaining_accounts);

    invoke_signed(
        &Instruction {
            program_id: candy_machine_program.key(),
            accounts: account_metas,
            // TODO. global::mint_nft instruction...
            data: vec![0xd3, 0x39, 0x06, 0xa7, 0x0f, 0xdb, 0x23, 0xfb],
        },
        &candy_machine_infos,
        &[&wallet_seeds],
    )?;

    // point back to the gumdrop authority
    let mut cm_config_data: &[u8] = &candy_machine_config.try_borrow_data()?;
    let cm_config = Config::try_deserialize(&mut cm_config_data)?;
    if cm_config.data.retain_authority {
        invoke_signed(
            &mpl_token_metadata::instruction::update_metadata_accounts(
                *token_metadata_program.key,
                *candy_machine_metadata.key,
                *distributor_wallet.key,
                Some(distributor.base),
                None,
                None,
            ),
            &[
                token_metadata_program.to_account_info(),
                candy_machine_metadata.to_account_info(),
                distributor_wallet.to_account_info(),
            ],
            &[&wallet_seeds],
        )?;
    }

    Ok(())
}

/// Accounts for [gumdrop::new_distributor].
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct NewDistributor<'info> {
    /// Base key of the distributor.
    pub base: Signer<'info>,

    /// [MerkleDistributor].
    #[account(
    init,
    seeds = [
    b"MerkleDistributor".as_ref(),
    base.key().to_bytes().as_ref()
    ],
    space = 8+97,
    bump,
    payer = payer
    )]
    pub distributor: Account<'info, MerkleDistributor>,

    /// Payer to create the distributor.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,
}

/// [gumdrop::close_distributor_token_acconut] accounts.
#[derive(Accounts)]
#[instruction(_bump: u8)]
pub struct CloseDistributorTokenAccount<'info> {
    /// Base key of the distributor.
    pub base: Signer<'info>,

    /// [MerkleDistributor].
    #[account(
    seeds = [
    b"MerkleDistributor".as_ref(),
    base.key().to_bytes().as_ref()
    ],
    bump = _bump,
    )]
    pub distributor: Account<'info, MerkleDistributor>,

    /// Distributor containing the tokens to distribute.
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,

    /// Account to send the claimed tokens to.
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,

    /// Who is receiving the remaining rent allocation.
    #[account(mut)]
    /// CHECK: just a destination
    pub receiver: AccountInfo<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,

    /// SPL [Token] program.
    pub token_program: Program<'info, Token>,
}

/// [gumdrop::close_distributor] accounts.
#[derive(Accounts)]
#[instruction(_bump: u8, _wallet_bump: u8)]
pub struct CloseDistributor<'info> {
    /// Base key of the distributor.
    pub base: Signer<'info>,

    /// [MerkleDistributor].
    #[account(
    seeds = [
    b"MerkleDistributor".as_ref(),
    base.key().to_bytes().as_ref()
    ],
    bump = _bump,
    mut,
    close = receiver,
    )]
    pub distributor: Account<'info, MerkleDistributor>,

    #[account(
    seeds = [
    b"Wallet".as_ref(),
    distributor.key().to_bytes().as_ref()
    ],
    bump = _wallet_bump,
    mut,
    )]
    /// CHECK: PDA Checked
    pub distributor_wallet: AccountInfo<'info>,

    /// Who is receiving the remaining tokens and rent allocations.
    /// CHECK: just a destination
    pub receiver: AccountInfo<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,

    /// SPL [Token] program.
    pub token_program: Program<'info, Token>,
}

/// [gumdrop::prove_claim] accounts.
#[derive(Accounts)]
#[instruction(
claim_prefix: Vec < u8 >,
claim_bump: u8,
index: u64,
_amount: u64,
_claimant_secret: Pubkey,
_resource: Pubkey,
resource_nonce: Vec < u8 >,
)]
pub struct ProveClaim<'info> {
    /// The [MerkleDistributor].
    #[account(mut)]
    pub distributor: Account<'info, MerkleDistributor>,

    /// Status of the claim.
    #[account(
    init,
    seeds = [
    claim_prefix.as_slice(),
    index.to_le_bytes().as_ref(),
    distributor.key().to_bytes().as_ref()
    ],
    bump,
    payer = payer,
    space = 8 // discriminator
    + 8   // amount
    + 8   // count
    + 32  // claimant
    + 32  // resource
    + 4 + resource_nonce.len() // resource_nonce vec
    )]
    pub claim_proof: Account<'info, ClaimProof>,

    /// Extra signer expected for claims
    pub temporal: Signer<'info>,

    /// Payer of the claim.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,
}

/// [gumdrop::claim] accounts.
#[derive(Accounts)]
#[instruction(claim_bump: u8, index: u64)]
pub struct Claim<'info> {
    /// The [MerkleDistributor].
    #[account(mut)]
    pub distributor: Account<'info, MerkleDistributor>,

    /// Status of the claim.
    #[account(
    init,
    seeds = [
    CLAIM_STATUS.as_ref(),
    index.to_le_bytes().as_ref(),
    distributor.key().to_bytes().as_ref()
    ],
    space = 8+49,
    bump,
    payer = payer
    )]
    pub claim_status: Account<'info, ClaimStatus>,

    /// Distributor containing the tokens to distribute.
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,

    /// Account to send the claimed tokens to.
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,

    /// Extra signer expected for claims
    pub temporal: Signer<'info>,

    /// Payer of the claim.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,

    /// SPL [Token] program.
    pub token_program: Program<'info, Token>,
}

/// [gumdrop::claim_candy] accounts.
#[derive(Accounts)]
#[instruction(_wallet_bump: u8, claim_bump: u8, index: u64)]
pub struct ClaimCandy<'info> {
    /// The [MerkleDistributor].
    #[account(mut)]
    pub distributor: Account<'info, MerkleDistributor>,

    /// The [MerkleDistributor] wallet
    #[account(
    seeds = [
    b"Wallet".as_ref(),
    distributor.key().to_bytes().as_ref()
    ],
    bump = _wallet_bump,
    mut
    )]
    /// CHECK: PDA check enforced
    pub distributor_wallet: AccountInfo<'info>,

    /// Status of the claim. Created on first invocation of this function
    #[account(
    seeds = [
    CLAIM_COUNT.as_ref(),
    index.to_le_bytes().as_ref(),
    distributor.key().to_bytes().as_ref()
    ],
    bump = claim_bump,
    mut,
    )]
    /// CHECK: PDA check enforced
    pub claim_count: AccountInfo<'info>,

    /// Extra signer expected for claims
    pub temporal: Signer<'info>,

    /// Payer of the claim. Should be `mint_authority` for `candy_machine_mint` and will be
    /// `update_authority` for `candy_machine_metadata`
    pub payer: Signer<'info>,

    /// Candy-machine Config
    /// CHECK: PDA check enforced in cpi
    pub candy_machine_config: AccountInfo<'info>,

    /// Candy-Machine. Verified through CPI
    #[account(mut)]
    /// CHECK: PDA check enforced in cpi
    pub candy_machine: AccountInfo<'info>,

    /// Candy-Machine-Wallet. Verified through CPI
    #[account(mut)]
    /// CHECK: PDA check enforced in cpi
    pub candy_machine_wallet: AccountInfo<'info>,

    /// Generated mint
    #[account(mut)]
    /// CHECK: PDA check enforced in cpi
    pub candy_machine_mint: AccountInfo<'info>,

    /// PDA of `candy_machine_mint`
    #[account(mut)]
    /// CHECK: PDA check enforced in cpi
    pub candy_machine_metadata: AccountInfo<'info>,

    /// PDA of `candy_machine_mint`
    #[account(mut)]
    /// CHECK: PDA check enforced in cpi
    pub candy_machine_master_edition: AccountInfo<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,

    /// SPL [Token] program.
    pub token_program: Program<'info, Token>,

    /// SPL [TokenMetadata] program.
    #[account(address = mpl_token_metadata::id())]
    /// CHECK: Address Checked
    pub token_metadata_program: AccountInfo<'info>,

    /// SPL [CandyMachine] program.
    /// CHECK: account checked in handler
    pub candy_machine_program: AccountInfo<'info>,

    rent: Sysvar<'info, Rent>,
    clock: Sysvar<'info, Clock>,
}

/// [gumdrop::claim_edition] accounts. Wrapper around
/// MintNewEditionFromMasterEditionViaToken
#[derive(Accounts)]
#[instruction(claim_bump: u8, index: u64)]
pub struct ClaimEdition<'info> {
    /// Given a token account containing the master edition token to prove authority, and a brand new non-metadata-ed mint with one token
    /// make a new Metadata + Edition that is a child of the master edition denoted by this authority token.
    ///   4. `[writable]` Edition pda to mark creation - will be checked for pre-existence. (pda of ['metadata', program id, master metadata mint id, 'edition', edition_number])
    ///   where edition_number is NOT the edition number you pass in args but actually edition_number = floor(edition/EDITION_MARKER_BIT_SIZE).
    ///   8. `[]` token account containing token from master metadata mint

    /// The [MerkleDistributor].
    #[account(mut)]
    pub distributor: Account<'info, MerkleDistributor>,

    /// Status of the claim. Created on first invocation of this function
    #[account(
    seeds = [
    CLAIM_COUNT.as_ref(),
    index.to_le_bytes().as_ref(),
    distributor.key().to_bytes().as_ref()
    ],
    bump = claim_bump,
    mut,
    )]
    /// CHECK: PDA Check enforced, and hashcheck
    pub claim_count: AccountInfo<'info>,

    /// Extra signer expected for claims
    pub temporal: Signer<'info>,

    /// Payer of the claim. Should be `mint_authority` for `candy_machine_mint` and will be
    /// `update_authority` for `candy_machine_metadata`
    pub payer: Signer<'info>,

    /// PDA of `metadata_new_mint`
    #[account(mut)]
    /// CHECK: Checked via CPI
    pub metadata_new_metadata: AccountInfo<'info>,

    /// PDA of `metadata_new_mint`
    #[account(mut)]
    /// CHECK: Checked via CPI
    pub metadata_new_edition: AccountInfo<'info>,

    /// PDA of `metadata_master_mint`
    #[account(mut)]
    /// CHECK: Checked via CPI
    pub metadata_master_edition: AccountInfo<'info>,

    /// Generated mint
    #[account(mut)]
    /// CHECK: Checked via CPI
    pub metadata_new_mint: AccountInfo<'info>,

    /// PDA of `metadata_master_mint` and edition number
    #[account(mut)]
    /// CHECK: Checked via CPI
    pub metadata_edition_mark_pda: AccountInfo<'info>,

    /// Mint authority for `metadata_new_mint`
    pub metadata_new_mint_authority: Signer<'info>,

    /// Owner of token account containing master token (#8)
    /// distributor

    /// Token account
    /// CHECK: Checked via CPI
    pub metadata_master_token_account: AccountInfo<'info>,

    /// Update authority for new metadata
    /// CHECK: Checked via CPI
    pub metadata_new_update_authority: AccountInfo<'info>,

    /// Master record metadata account
    /// CHECK: Checked via CPI
    pub metadata_master_metadata: AccountInfo<'info>,

    /// Master metadata mint account
    /// CHECK: Checked via CPI
    pub metadata_master_mint: AccountInfo<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,

    /// SPL [Token] program.
    pub token_program: Program<'info, Token>,

    /// SPL [TokenMetadata] program.
    #[account(address = mpl_token_metadata::id())]
    /// CHECK: Address Check
    pub token_metadata_program: AccountInfo<'info>,

    rent: Sysvar<'info, Rent>,
}

/// [gumdrop::claim_candy_proven] accounts.
#[derive(Accounts)]
#[instruction(wallet_bump: u8, claim_bump: u8, index: u64)]
pub struct ClaimCandyProven<'info> {
    /// The [MerkleDistributor].
    #[account(mut)]
    pub distributor: Account<'info, MerkleDistributor>,

    /// The [MerkleDistributor] wallet
    #[account(
    seeds = [
    b"Wallet".as_ref(),
    distributor.key().to_bytes().as_ref()
    ],
    bump = wallet_bump,
    mut
    )]
    /// CHECK: PDA checked
    pub distributor_wallet: AccountInfo<'info>,

    /// Status of the claim. Created with prove_claim
    #[account(
    seeds = [
    CLAIM_COUNT.as_ref(),
    index.to_le_bytes().as_ref(),
    distributor.key().to_bytes().as_ref()
    ],
    bump = claim_bump,
    mut,
    )]
    pub claim_proof: Account<'info, ClaimProof>,

    /// Payer of the claim. Should be `mint_authority` for `candy_machine_mint` and will be
    /// `update_authority` for `candy_machine_metadata`
    pub payer: Signer<'info>,

    /// Candy-machine Config
    /// CHECK: Checked via CPI
    pub candy_machine_config: AccountInfo<'info>,

    /// Candy-Machine. Verified through CPI
    #[account(mut)]
    /// CHECK: Checked via CPI
    pub candy_machine: AccountInfo<'info>,

    /// Candy-Machine-Wallet. Verified through CPI
    #[account(mut)]
    /// CHECK: Checked via CPI
    pub candy_machine_wallet: AccountInfo<'info>,

    /// Generated mint
    #[account(mut)]
    /// CHECK: Checked via CPI
    pub candy_machine_mint: AccountInfo<'info>,

    /// PDA of `candy_machine_mint`
    #[account(mut)]
    /// CHECK: Checked via CPI
    pub candy_machine_metadata: AccountInfo<'info>,

    /// PDA of `candy_machine_mint`
    #[account(mut)]
    /// CHECK: Checked via CPI
    pub candy_machine_master_edition: AccountInfo<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,

    /// SPL [Token] program.
    pub token_program: Program<'info, Token>,

    /// SPL [TokenMetadata] program.
    #[account(address = mpl_token_metadata::id())]
    /// CHECK:
    pub token_metadata_program: AccountInfo<'info>,

    /// SPL [CandyMachine] program.
    /// CHECK: account checked in handler
    pub candy_machine_program: AccountInfo<'info>,

    rent: Sysvar<'info, Rent>,
    clock: Sysvar<'info, Clock>,
}

/// [gumdrop::recover_update_authority] accounts.
#[derive(Accounts)]
#[instruction(_bump: u8, wallet_bump: u8)]
pub struct RecoverUpdateAuthority<'info> {
    /// Base key of the distributor.
    pub base: Signer<'info>,

    /// [MerkleDistributor].
    #[account(
    seeds = [
    b"MerkleDistributor".as_ref(),
    base.key().to_bytes().as_ref()
    ],
    bump = _bump,
    )]
    pub distributor: Account<'info, MerkleDistributor>,

    /// The [MerkleDistributor] wallet
    #[account(
    seeds = [
    b"Wallet".as_ref(),
    distributor.key().to_bytes().as_ref()
    ],
    bump = wallet_bump,
    )]
    /// CHECK: Checked via PDA seed and base sign
    pub distributor_wallet: AccountInfo<'info>,

    /// New update authority
    /// CHECK: No need to check this is input
    pub new_update_authority: AccountInfo<'info>,

    /// Metadata account to update authority for
    #[account(mut)]
    /// CHECK: Checked via CPI
    pub metadata: AccountInfo<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,

    /// SPL [TokenMetadata] program.
    #[account(address = mpl_token_metadata::id())]
    /// CHECK: Address Checked
    pub token_metadata_program: AccountInfo<'info>,
}

/// State for the account which distributes tokens.
#[account]
#[derive(Default)]
pub struct MerkleDistributor {
    /// Base key used to generate the PDA.
    pub base: Pubkey,
    /// Bump seed.
    pub bump: u8,

    /// The 256-bit merkle root.
    pub root: [u8; 32],

    /// Third-party signer expected on claims. Verified by OTP with off-chain distribution method
    pub temporal: Pubkey,
}

#[account]
#[derive(Default)]
pub struct ClaimStatus {
    /// If true, the tokens have been claimed.
    pub is_claimed: bool,
    /// Authority that claimed the tokens.
    pub claimant: Pubkey,
    /// When the tokens were claimed.
    pub claimed_at: i64,
    /// Amount of tokens claimed.
    pub amount: u64,
}

#[account]
#[derive(Default)]
pub struct ClaimCount {
    /// Number of NFTs claimed. Compared versus `amount` in merkle tree data / proof
    pub count: u64,
    /// Authority that claimed the tokens.
    pub claimant: Pubkey,
}

/// Allows for proof and candy minting in separate transactions to avoid transaction-size limit.
///
/// Used for all resources (tokens, candy claims, and edition mints)
#[account]
#[derive(Default)]
pub struct ClaimProof {
    /// Total number of NFTs that can be claimed
    pub amount: u64,
    /// Number of NFTs claimed. Compared versus `amount` in merkle tree data / proof
    pub count: u64,
    /// Authority that claimed the tokens.
    pub claimant: Pubkey,
    /// Resource allocated for this gumdrop. There should only be 1 per gumdrop
    pub resource: Pubkey,
    pub resource_nonce: Vec<u8>,
}

/// Emitted when tokens are claimed.
#[event]
pub struct ClaimedEvent {
    /// Index of the claim.
    pub index: u64,
    /// User that claimed.
    pub claimant: Pubkey,
    /// Amount of tokens to distribute.
    pub amount: u64,
}

#[error_code]
pub enum GumdropError {
    #[msg("Invalid Merkle proof.")]
    InvalidProof,
    #[msg("Drop already claimed.")]
    DropAlreadyClaimed,
    #[msg("Account is not authorized to execute this instruction")]
    Unauthorized,
    #[msg("Token account owner did not match intended owner")]
    OwnerMismatch,
    #[msg("Temporal signer did not match distributor")]
    TemporalMismatch,
    #[msg("Numerical Overflow")]
    NumericalOverflow,
    #[msg("Invalid Claim Bump")]
    InvalidClaimBump,
    #[msg("Gumdrop only supports the official Metaplex Candy machine contracts")]
    MustUseOfficialCandyMachine,
}

#[account]
#[derive(Default)]
pub struct CandyMachine {
    pub authority: Pubkey,
    pub wallet: Pubkey,
    pub token_mint: Option<Pubkey>,
    pub config: Pubkey,
    pub data: CandyMachineData,
    pub items_redeemed: u64,
    pub bump: u8,
}

#[account]
#[derive(Default)]
pub struct Config {
    pub authority: Pubkey,
    pub data: ConfigData,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ConfigData {
    pub uuid: String,
    /// The symbol for the asset
    pub symbol: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    pub creators: Vec<Creator>,
    pub max_supply: u64,
    pub is_mutable: bool,
    pub retain_authority: bool,
    pub max_number_of_lines: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct CandyMachineData {
    pub uuid: String,
    pub price: u64,
    pub items_available: u64,
    pub go_live_date: Option<i64>,
}