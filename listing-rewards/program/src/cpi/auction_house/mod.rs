use anchor_lang::prelude::*;
use mpl_auction_house::id;
use solana_program::instruction::Instruction;

pub struct AuctioneerInstructionArgs<T> {
    pub accounts: T,
    pub instruction_data: Vec<u8>,
    pub auctioneer_authority: Pubkey,
}

pub fn make_auctioneer_instruction<'a, T: ToAccountInfos<'a> + ToAccountMetas>(
    AuctioneerInstructionArgs {
        accounts,
        instruction_data,
        auctioneer_authority,
    }: AuctioneerInstructionArgs<T>,
) -> (Instruction, Vec<AccountInfo<'a>>) {
    let account_infos = accounts.to_account_infos();

    (
        Instruction {
            program_id: id(),
            data: instruction_data,
            accounts: accounts
                .to_account_metas(None)
                .into_iter()
                .zip(account_infos.clone())
                .map(|mut pair| {
                    pair.0.is_signer = pair.1.is_signer;
                    if pair.0.pubkey == auctioneer_authority {
                        pair.0.is_signer = true;
                    }
                    pair.0
                })
                .collect(),
        },
        account_infos,
    )
}
