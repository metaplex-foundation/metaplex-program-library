use anchor_lang::{
    prelude::{Pubkey, Rent},
    InstructionData, ToAccountMetas,
};
use mpl_token_metadata::{instruction::create_metadata_accounts, pda::find_metadata_account};
use solana_program_test::ProgramTest;
use solana_sdk::{
    instruction::Instruction, program_pack::Pack, signature::Keypair, signer::Signer,
    system_instruction::create_account, transaction::Transaction,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_token::{
    instruction::{approve, initialize_mint, mint_to, revoke},
    state::Mint,
};
use test_utils::{
    find_entangled_pair, find_escrow_a, find_escrow_b, find_master_edition_address,
    instructions_to_mint_an_nft,
};

// #[tokio::test]
async fn _lifecycle_test() {
    const TREASURY_MINT: &str = "So11111111111111111111111111111111111111112";
    const RENT_SYSVAR_ADDRESS: &str = "SysvarRent111111111111111111111111111111111";
    const SYSTEM_PROGRAM_ADDRESS: &str = "11111111111111111111111111111111";

    let mut program_test = ProgramTest::default();
    program_test.add_program("mpl_token_metadata", mpl_token_metadata::id(), None);
    program_test.add_program("mpl_token_entangler", mpl_token_entangler::id(), None);

    let context = program_test.start_with_context().await;
    let mut banks_client = context.banks_client;
    let payer = context.payer;
    let rent = banks_client.get_sysvar::<Rent>().await.unwrap();

    let mint_a = Keypair::new();
    let mint_a_tx = Transaction::new_signed_with_payer(
        &instructions_to_mint_an_nft(payer.pubkey(), mint_a.pubkey(), &rent),
        Some(&payer.pubkey()),
        &[&payer, &mint_a],
        context.last_blockhash,
    );
    banks_client.process_transaction(mint_a_tx).await.unwrap();

    let mint_b = Keypair::new();
    let mint_b_tx = Transaction::new_signed_with_payer(
        &instructions_to_mint_an_nft(payer.pubkey(), mint_b.pubkey(), &rent),
        Some(&payer.pubkey()),
        &[&payer, &mint_b],
        context.last_blockhash,
    );
    banks_client.process_transaction(mint_b_tx).await.unwrap();

    let entangled_pair = find_entangled_pair(mint_a.pubkey(), mint_b.pubkey());
    let reverse_pair = find_entangled_pair(mint_b.pubkey(), mint_a.pubkey());

    let escrow_a = find_escrow_a(mint_a.pubkey(), mint_b.pubkey());
    let escrow_b = find_escrow_b(mint_a.pubkey(), mint_b.pubkey());

    {
        // CreateEntangledPair (marathon)
        let transfer_authority = Keypair::new();
        let token_b = get_associated_token_address(&payer.pubkey(), &mint_b.pubkey());

        let accounts = mpl_token_entangler::accounts::CreateEntangledPair {
            payer: payer.pubkey(),
            authority: payer.pubkey(),
            treasury_mint: TREASURY_MINT.parse().unwrap(),
            transfer_authority: transfer_authority.pubkey(),
            entangled_pair: entangled_pair.0,
            reverse_entangled_pair: reverse_pair.0,
            mint_a: mint_a.pubkey(),
            mint_b: mint_b.pubkey(),
            token_a_escrow: escrow_a.0,
            token_b_escrow: escrow_b.0,
            metadata_a: find_metadata_account(&mint_a.pubkey()).0,
            metadata_b: find_metadata_account(&mint_b.pubkey()).0,
            edition_a: find_master_edition_address(mint_a.pubkey()),
            edition_b: find_master_edition_address(mint_b.pubkey()),
            token_b,
            token_program: spl_token::id(),
            rent: RENT_SYSVAR_ADDRESS.parse().unwrap(),
            system_program: SYSTEM_PROGRAM_ADDRESS.parse().unwrap(),
        };

        let instruction = mpl_token_entangler::instruction::CreateEntangledPair {
            bump: entangled_pair.1,
            _reverse_bump: reverse_pair.1,
            token_a_escrow_bump: escrow_a.1,
            token_b_escrow_bump: escrow_b.1,
            price: 1,
            pays_every_time: true,
        };

        let instructions = [
            approve(
                &spl_token::id(),
                &token_b,
                &transfer_authority.pubkey(),
                &payer.pubkey(),
                &[],
                1,
            )
            .unwrap(),
            Instruction {
                program_id: mpl_token_entangler::id(),
                accounts: accounts.to_account_metas(None),
                data: instruction.data(),
            },
            revoke(
                &spl_token::id(),
                &token_b,
                &payer.pubkey(),
                &[], //
            )
            .unwrap(),
        ];

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[&payer, &transfer_authority],
            context.last_blockhash,
        );

        banks_client.process_transaction(tx).await.unwrap();
    }

    {
        // SWAP'em
        let payment_transfer_authority = Keypair::new();
        let transfer_authority = Keypair::new();
        let token_a = get_associated_token_address(&payer.pubkey(), &mint_a.pubkey());
        let token_b = get_associated_token_address(&payer.pubkey(), &mint_b.pubkey());

        let accounts = mpl_token_entangler::accounts::Swap {
            treasury_mint: TREASURY_MINT.parse().unwrap(),
            payer: payer.pubkey(),
            payment_account: payer.pubkey(),
            payment_transfer_authority: payment_transfer_authority.pubkey(),
            token: token_a,
            token_mint: mint_a.pubkey(),
            replacement_token_metadata: find_metadata_account(&mint_b.pubkey()).0,
            replacement_token_mint: mint_b.pubkey(),
            replacement_token: token_b,
            transfer_authority: transfer_authority.pubkey(),
            token_a_escrow: escrow_a.0,
            token_b_escrow: escrow_b.0,
            entangled_pair: entangled_pair.0,
            token_program: spl_token::id(),
            system_program: SYSTEM_PROGRAM_ADDRESS.parse().unwrap(),
            ata_program: spl_associated_token_account::id(),
            rent: RENT_SYSVAR_ADDRESS.parse().unwrap(),
        };

        let instruction = mpl_token_entangler::instruction::Swap {};

        let instructions = [
            approve(
                &spl_token::id(),
                &token_a,
                &transfer_authority.pubkey(),
                &payer.pubkey(),
                &[],
                1,
            )
            .unwrap(),
            Instruction {
                program_id: mpl_token_entangler::id(),
                accounts: accounts.to_account_metas(None),
                data: instruction.data(),
            },
            revoke(
                &spl_token::id(),
                &token_a,
                &payer.pubkey(),
                &[], //
            )
            .unwrap(),
        ];

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[&payer, &transfer_authority],
            context.last_blockhash,
        );

        banks_client.process_transaction(tx).await.unwrap();
    }

    {
        // SWAP'em again
        let payment_transfer_authority = Keypair::new();
        let transfer_authority = Keypair::new();
        let token_a = get_associated_token_address(&payer.pubkey(), &mint_a.pubkey());
        let token_b = get_associated_token_address(&payer.pubkey(), &mint_b.pubkey());

        let accounts = mpl_token_entangler::accounts::Swap {
            treasury_mint: TREASURY_MINT.parse().unwrap(),
            payer: payer.pubkey(),
            payment_account: payer.pubkey(),
            payment_transfer_authority: payment_transfer_authority.pubkey(),
            token: token_b,
            token_mint: mint_b.pubkey(),
            replacement_token_metadata: find_metadata_account(&mint_a.pubkey()).0,
            replacement_token_mint: mint_a.pubkey(),
            replacement_token: token_a,
            transfer_authority: transfer_authority.pubkey(),
            token_a_escrow: escrow_a.0,
            token_b_escrow: escrow_b.0,
            entangled_pair: entangled_pair.0,
            token_program: spl_token::id(),
            system_program: SYSTEM_PROGRAM_ADDRESS.parse().unwrap(),
            ata_program: spl_associated_token_account::id(),
            rent: RENT_SYSVAR_ADDRESS.parse().unwrap(),
        };

        let instruction = mpl_token_entangler::instruction::Swap {};

        let instructions = [
            approve(
                &spl_token::id(),
                &token_b,
                &transfer_authority.pubkey(),
                &payer.pubkey(),
                &[],
                1,
            )
            .unwrap(),
            Instruction {
                program_id: mpl_token_entangler::id(),
                accounts: accounts.to_account_metas(None),
                data: instruction.data(),
            },
            revoke(
                &spl_token::id(),
                &token_b,
                &payer.pubkey(),
                &[], //
            )
            .unwrap(),
        ];

        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[&payer, &transfer_authority],
            context.last_blockhash,
        );

        banks_client.process_transaction(tx).await.unwrap();
    }
}

#[allow(unused)]
mod test_utils {
    use mpl_token_metadata::instruction::create_master_edition_v3;

    use crate::*;

    pub fn instructions_to_mint_an_nft(
        payer: Pubkey,
        mint: Pubkey,
        rent: &Rent,
    ) -> Vec<solana_sdk::instruction::Instruction> {
        vec![
            create_account(
                &payer,
                &mint,
                rent.minimum_balance(Mint::LEN),
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            initialize_mint(&spl_token::id(), &mint, &payer, Some(&payer), 0).unwrap(),
            create_associated_token_account(&payer, &payer, &mint),
            create_metadata_accounts(
                mpl_token_metadata::id(),
                find_metadata_account(&mint).0,
                mint,
                payer,
                payer,
                payer,
                "a-name".to_owned(),
                "a-symbol".to_owned(),
                "a-uri".to_owned(),
                None,
                500,
                true,
                true,
            ),
            mint_to(
                &spl_token::id(),
                &mint,
                &get_associated_token_address(&payer, &mint),
                &payer,
                &[&payer],
                1,
            )
            .unwrap(),
            create_master_edition_v3(
                mpl_token_metadata::id(),
                find_master_edition_address(mint),
                mint,
                payer,
                payer,
                find_metadata_account(&mint).0,
                payer,
                Some(1),
            ),
        ]
    }

    pub fn find_entangled_pair(mint_a: Pubkey, mint_b: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                "token_entangler".as_bytes(),
                mint_a.as_ref(),
                mint_b.as_ref(),
            ],
            &mpl_token_entangler::id(),
        )
    }

    pub fn find_escrow_a(mint_a: Pubkey, mint_b: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                "token_entangler".as_bytes(),
                mint_a.as_ref(),
                mint_b.as_ref(),
                "escrow".as_bytes(),
                "A".as_bytes(),
            ],
            &mpl_token_entangler::id(),
        )
    }

    pub fn find_escrow_b(mint_a: Pubkey, mint_b: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                "token_entangler".as_bytes(),
                mint_a.as_ref(),
                mint_b.as_ref(),
                "escrow".as_bytes(),
                "B".as_bytes(),
            ],
            &mpl_token_entangler::id(),
        )
    }

    pub fn find_master_edition_address(mint: Pubkey) -> Pubkey {
        let (address, _bump) = Pubkey::find_program_address(
            &[
                mpl_token_metadata::state::PREFIX.as_bytes(),
                mpl_token_metadata::id().as_ref(),
                mint.as_ref(),
                "edition".as_ref(),
            ],
            &mpl_token_metadata::id(),
        );
        address
    }
}
