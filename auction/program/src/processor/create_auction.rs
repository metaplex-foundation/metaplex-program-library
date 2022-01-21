use mem::size_of;

use crate::{
    errors::AuctionError,
    processor::{
        AuctionData, AuctionDataExtended, AuctionName, AuctionState, Bid, BidState, PriceFloor,
        WinnerLimit, BASE_AUCTION_DATA_SIZE, MAX_AUCTION_DATA_EXTENDED_SIZE,
    },
    utils::{assert_derivation, assert_owned_by, create_or_allocate_account_raw},
    EXTENDED, PREFIX,
};

use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        clock::UnixTimestamp,
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    std::mem,
};

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CreateAuctionArgs {
    /// How many winners are allowed for this auction. See AuctionData.
    pub winners: WinnerLimit,
    /// End time is the cut-off point that the auction is forced to end by. See AuctionData.
    pub end_auction_at: Option<UnixTimestamp>,
    /// Gap time is how much time after the previous bid where the auction ends. See AuctionData.
    pub end_auction_gap: Option<UnixTimestamp>,
    /// Token mint for the SPL token used for bidding.
    pub token_mint: Pubkey,
    /// Authority
    pub authority: Pubkey,
    /// The resource being auctioned. See AuctionData.
    pub resource: Pubkey,
    /// Set a price floor.
    pub price_floor: PriceFloor,
    /// Add a tick size increment
    pub tick_size: Option<u64>,
    /// Add a minimum percentage increase each bid must meet.
    pub gap_tick_size_percentage: Option<u8>,
}

struct Accounts<'a, 'b: 'a> {
    auction: &'a AccountInfo<'b>,
    auction_extended: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
    rent: &'a AccountInfo<'b>,
    system: &'a AccountInfo<'b>,
}

fn parse_accounts<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let account_iter = &mut accounts.iter();
    let accounts = Accounts {
        payer: next_account_info(account_iter)?,
        auction: next_account_info(account_iter)?,
        auction_extended: next_account_info(account_iter)?,
        rent: next_account_info(account_iter)?,
        system: next_account_info(account_iter)?,
    };
    Ok(accounts)
}

pub fn create_auction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: CreateAuctionArgs,
    instant_sale_price: Option<u64>,
    name: Option<AuctionName>,
    bid_type: Option<u8>,
) -> ProgramResult {
    msg!("+ Processing CreateAuction");
    let accounts = parse_accounts(program_id, accounts)?;

    let auction_path = [
        PREFIX.as_bytes(),
        program_id.as_ref(),
        &args.resource.to_bytes(),
    ];

    // Derive the address we'll store the auction in, and confirm it matches what we expected the
    // user to provide.
    let (auction_key, bump) = Pubkey::find_program_address(&auction_path, program_id);
    if auction_key != *accounts.auction.key {
        return Err(AuctionError::InvalidAuctionAccount.into());
    }
    // The data must be large enough to hold at least the number of winners.
    let auction_size = match args.winners {
        WinnerLimit::Capped(n) => {
            mem::size_of::<Bid>() * BidState::max_array_size_for(n) + BASE_AUCTION_DATA_SIZE
        }
        WinnerLimit::Unlimited(_) => BASE_AUCTION_DATA_SIZE,
    };

    let bidType: u64 = match bid_type {
        Some(v) => v as u64,
        None => 0,
    };

    let bid_state = match args.winners {
        WinnerLimit::Capped(n) => match bidType {
            0 => BidState::new_english(n),
            2 => BidState::new_dutch(n),
            _ => BidState::new_english(n),
        },
        WinnerLimit::Unlimited(_) => BidState::new_open_edition(),
    };

    let mut decline_interval = 0;
    let lamp = 1000000000;
    let decline_rate = 2 * lamp;
    let mut send_decrease_rate: u64 = 0;

    if (bidType == 2) {
        let price_ceiling: u64 = match instant_sale_price {
            Some(v) => v as u64,
            None => 0,
        };

        let price_floor = match args.price_floor {
            PriceFloor::MinimumPrice(v) => v[0],
            _ => 0,
        };

        let decrease_value: f64 =
            price_ceiling as f64 / lamp as f64 - price_floor as f64 / lamp as f64;

        let act_dec_rate: f64 = decline_rate as f64 / lamp as f64;

        //decrease/minute
        let decline_value: f64 = decrease_value * (act_dec_rate / 100.0);

        send_decrease_rate = (decline_value * lamp as f64) as u64;

        //Considering the toal time of dutch auction to be 180 minutes:
        msg!("The decline value {}", decline_value);

        //Calculating the decline_interval on the basis of decline value
        decline_interval = (((180.0 / decrease_value) * decline_value) * lamp as f64) as u64;

        // 1 minute = 10^9

        if price_ceiling < 0 {
            msg!("Ceiling price Price is either not set, or is set less than 0");
            return Err(AuctionError::CeilingPriceMandatoryDuctchAuction.into());
        }

        if price_ceiling < price_floor as u64 {
            msg!("Ceiling price can never be less than Floor price");
            return Err(AuctionError::CeilingPriceLessThanFloorPrice.into());
        }
    }

    if let Some(gap_tick) = args.gap_tick_size_percentage {
        if gap_tick > 100 {
            return Err(AuctionError::InvalidGapTickSizePercentage.into());
        }
    }

    // Create auction account with enough space for a winner tracking.
    create_or_allocate_account_raw(
        *program_id,
        accounts.auction,
        accounts.rent,
        accounts.system,
        accounts.payer,
        auction_size,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            &args.resource.to_bytes(),
            &[bump],
        ],
    )?;

    let auction_ext_bump = assert_derivation(
        program_id,
        accounts.auction_extended,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            &args.resource.to_bytes(),
            EXTENDED.as_bytes(),
        ],
    )?;

    create_or_allocate_account_raw(
        *program_id,
        accounts.auction_extended,
        accounts.rent,
        accounts.system,
        accounts.payer,
        MAX_AUCTION_DATA_EXTENDED_SIZE,
        &[
            PREFIX.as_bytes(),
            program_id.as_ref(),
            &args.resource.to_bytes(),
            EXTENDED.as_bytes(),
            &[auction_ext_bump],
        ],
    )?;

    // Configure extended
    AuctionDataExtended {
        total_uncancelled_bids: 0,
        tick_size: args.tick_size,
        gap_tick_size_percentage: args.gap_tick_size_percentage,
        instant_sale_price,
        name,
        decrease_rate: Some(send_decrease_rate),
        decrease_interval: Some(decline_interval),
        auction_start_time: None,
    }
    .serialize(&mut *accounts.auction_extended.data.borrow_mut())?;

    // Configure Auction.
    AuctionData {
        authority: args.authority,
        bid_state: bid_state,
        end_auction_at: args.end_auction_at,
        end_auction_gap: args.end_auction_gap,
        ended_at: None,
        last_bid: None,
        price_floor: args.price_floor,
        state: AuctionState::create(),
        token_mint: args.token_mint,
    }
    .serialize(&mut *accounts.auction.data.borrow_mut())?;

    Ok(())
}
