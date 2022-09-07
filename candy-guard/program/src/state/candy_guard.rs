use anchor_lang::{prelude::*, AnchorDeserialize};

use crate::guards::*;
use mpl_candy_guard_derive::GuardSet;

// Bytes offset for the start of the data section:
//     8 (discriminator)
//  + 32 (base)
//  +  1 (bump)
//  + 32 (authority)
pub const DATA_OFFSET: usize = 8 + 32 + 1 + 32;

#[account]
#[derive(Default)]
pub struct CandyGuard {
    // Base key used to generate the PDA
    pub base: Pubkey,
    // Bump seed
    pub bump: u8,
    // Authority of the guard
    pub authority: Pubkey,
    // after this there is a flexible amount of data to serialize
    // data (CandyGuardData struct) of the available guards; the size
    // of the data is adjustable as new guards are implemented (the
    // account is resized using realloc)
    //
    // available guards:
    //  1) bot tax
    //  2) live date
    //  3) lamports
    //  4) spltoken
    //  5) third party signer
    //  6) whitelist
    //  7) gatekeeper
    //  8) end settings
    //  9) allow list
    // 10) mint limit
    // 11) nft payment
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CandyGuardData {
    pub default: GuardSet,
    pub groups: Option<Vec<GuardSet>>,
}

impl CandyGuardData {
    /// Serialize the candy guard data into the specified data array.
    pub fn save(&self, data: &mut [u8]) -> Result<()> {
        let mut cursor = 0;

        // saves the 'default' guard set
        let _ = self.default.to_data(data)?;
        cursor += self.default.size();

        // stores the number of 'groups' guard set
        let group_counter = if let Some(groups) = &self.groups {
            groups.len() as u32
        } else {
            0
        };
        data[cursor..cursor + 4].copy_from_slice(&u32::to_le_bytes(group_counter));
        cursor += 4;

        // saves each individual 'groups' guard set
        if let Some(groups) = &self.groups {
            for group in groups {
                let _ = group.to_data(&mut data[cursor..])?;
                cursor += group.size();
            }
        }

        Ok(())
    }

    /// Deserializes the guards. Only attempts the deserialization of individuals guards
    /// if the data slice is large enough.
    pub fn load(data: &[u8]) -> Result<Self> {
        let (default, _) = GuardSet::from_data(data)?;
        let mut cursor = default.size();

        let group_counter = u32::from_le_bytes(*arrayref::array_ref![data, cursor, 4]);
        cursor += 4;

        let groups = if group_counter > 0 {
            let mut groups = Vec::with_capacity(group_counter as usize);

            for _i in 0..group_counter {
                let (group, _) = GuardSet::from_data(&data[cursor..])?;
                cursor += group.size();
                groups.push(group);
            }

            Some(groups)
        } else {
            None
        };

        Ok(Self { default, groups })
    }

    pub fn active_set(data: &[u8]) -> Result<GuardSet> {
        // root guard set
        let (mut root, _) = GuardSet::from_data(data)?;
        let mut cursor = root.size();

        // number of groups
        let group_counter = u32::from_le_bytes(*arrayref::array_ref![data, cursor, 4]);
        cursor += 4;

        // 1 - timestamp
        // 2 - data offset
        let mut active = (0, 0);

        if group_counter > 0 {
            let clock = Clock::get()?;

            // determines the active guard set based on the live date: the active one has
            // the latest live data that is earlier than the current timestamp
            for _i in 0..group_counter {
                let features = u64::from_le_bytes(*arrayref::array_ref![data, cursor, 8]);

                if LiveDate::is_enabled(features) {
                    let inner_cursor = 8 + if BotTax::is_enabled(features) {
                        BotTax::size()
                    } else {
                        0
                    };

                    let live_date = LiveDate::load(data, inner_cursor)?.unwrap();

                    if let Some(date) = live_date.date {
                        if date > active.0 && date < clock.unix_timestamp {
                            active.0 = date;
                            active.1 = cursor;
                        }
                    }
                }

                cursor += GuardSet::bytes_count(features);
            }

            // deserialize the active group

            (_, cursor) = active;

            if cursor > 0 {
                let (group, _) = GuardSet::from_data(&data[cursor..])?;
                root.merge(group);
            }
        }

        Ok(root)
    }

    pub fn size(&self) -> usize {
        let mut size = DATA_OFFSET + self.default.size();
        size += 4; // u32 (number of groups)

        if let Some(groups) = &self.groups {
            size += groups.iter().map(|group| group.size()).sum::<usize>();
        }

        size
    }
}

#[derive(GuardSet, AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GuardSet {
    /// Last instruction check and bot tax (penalty for invalid transactions).
    pub bot_tax: Option<BotTax>,
    /// Lamports guard (set the price for the mint in lamports).
    pub lamports: Option<Lamports>,
    /// Spl-token guard (set the price for the mint in spl-token amount).
    pub spl_token: Option<SplToken>,
    /// Live data guard (controls when minting is allowed).
    pub live_date: Option<LiveDate>,
    /// Third party signer guard.
    pub third_party_signer: Option<ThirdPartySigner>,
    /// Whitelist guard (whitelist mint settings).
    pub whitelist: Option<Whitelist>,
    /// Gatekeeper guard
    pub gatekeeper: Option<Gatekeeper>,
    /// End settings guard
    pub end_settings: Option<EndSettings>,
    /// Allow list guard
    pub allow_list: Option<AllowList>,
    /// Mint limit guard
    pub mint_limit: Option<MintLimit>,
    /// NFT Payment
    pub nft_payment: Option<NftPayment>,
}
