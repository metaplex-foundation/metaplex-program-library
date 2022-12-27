use anchor_lang::prelude::Pubkey;
use anyhow::{anyhow, Result};
use dateparser::DateTimeUtc;
use serde::{Deserialize, Serialize};

use super::{data::price_as_lamports, to_pubkey, to_string};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CandyGuardData {
    pub default: GuardSet,
    pub groups: Option<Vec<Group>>,
}

impl CandyGuardData {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::state::CandyGuardData> {
        let groups = if let Some(groups) = &self.groups {
            let mut group_vec = Vec::with_capacity(groups.len());

            for group in groups {
                group_vec.push(group.to_guard_format()?);
            }

            Some(group_vec)
        } else {
            None
        };

        Ok(mpl_candy_guard::state::CandyGuardData {
            default: self.default.to_guard_format()?,
            groups,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Group {
    pub label: String,
    pub guards: GuardSet,
}

impl Group {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::state::Group> {
        Ok(mpl_candy_guard::state::Group {
            label: self.label.clone(),
            guards: self.guards.to_guard_format()?,
        })
    }
}

/// The set of guards available.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GuardSet {
    /// Last instruction check and bot tax (penalty for invalid transactions).
    pub bot_tax: Option<BotTax>,
    /// Sol payment guard (set the price for the mint in lamports).
    pub sol_payment: Option<SolPayment>,
    /// Token payment guard (set the price for the mint in spl-token amount).
    pub token_payment: Option<TokenPayment>,
    /// Start data guard (controls when minting is allowed).
    pub start_date: Option<StartDate>,
    /// Third party signer guard.
    pub third_party_signer: Option<ThirdPartySigner>,
    /// Token gate guard (restricrt access to holders of a specific token).
    pub token_gate: Option<TokenGate>,
    /// Gatekeeper guard
    pub gatekeeper: Option<Gatekeeper>,
    /// End date guard
    pub end_date: Option<EndDate>,
    /// Allow list guard
    pub allow_list: Option<AllowList>,
    /// Mint limit guard
    pub mint_limit: Option<MintLimit>,
    /// NFT Payment
    pub nft_payment: Option<NftPayment>,
    /// Redeemed amount guard
    pub redeemed_amount: Option<RedeemedAmount>,
    /// Address gate (check access against a specified address)
    pub address_gate: Option<AddressGate>,
    /// NFT gate guard (check access based on holding a specified NFT)
    pub nft_gate: Option<NftGate>,
    /// NFT burn guard (burn a specified NFT)
    pub nft_burn: Option<NftBurn>,
    /// Token burn guard (burn a specified amount of spl-token)
    pub token_burn: Option<TokenBurn>,
    /// Freeze sol payment guard (set the price for the mint in lamports with a freeze period).
    pub freeze_sol_payment: Option<FreezeSolPayment>,
    /// Freeze token payment guard (set the price for the mint in spl-token amount with a freeze period).
    pub freeze_token_payment: Option<FreezeTokenPayment>,
}

impl GuardSet {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::GuardSet> {
        // bot tax
        let bot_tax = if let Some(bot_tax) = &self.bot_tax {
            Some(bot_tax.to_guard_format()?)
        } else {
            None
        };
        // sol payment
        let sol_payment = if let Some(sol_payment) = &self.sol_payment {
            Some(sol_payment.to_guard_format()?)
        } else {
            None
        };
        // token payment
        let token_payment = if let Some(token_payment) = &self.token_payment {
            Some(token_payment.to_guard_format()?)
        } else {
            None
        };
        // start_date
        let start_date = if let Some(start_date) = &self.start_date {
            Some(start_date.to_guard_format()?)
        } else {
            None
        };
        // third party signer
        let third_party_signer = if let Some(third_party_signer) = &self.third_party_signer {
            Some(third_party_signer.to_guard_format()?)
        } else {
            None
        };
        // token gate
        let token_gate = if let Some(token_gate) = &self.token_gate {
            Some(token_gate.to_guard_format()?)
        } else {
            None
        };
        // gatekeeper
        let gatekeeper = if let Some(gatekeeper) = &self.gatekeeper {
            Some(gatekeeper.to_guard_format()?)
        } else {
            None
        };
        // end date
        let end_date = if let Some(end_date) = &self.end_date {
            Some(end_date.to_guard_format()?)
        } else {
            None
        };
        // allow list
        let allow_list = if let Some(allow_list) = &self.allow_list {
            Some(allow_list.to_guard_format()?)
        } else {
            None
        };
        // mint limit
        let mint_limit = if let Some(mint_limit) = &self.mint_limit {
            Some(mint_limit.to_guard_format()?)
        } else {
            None
        };
        // nft payment
        let nft_payment = if let Some(nft_payment) = &self.nft_payment {
            Some(nft_payment.to_guard_format()?)
        } else {
            None
        };
        // redeemed amount
        let redeemed_amount = if let Some(redeemed_amount) = &self.redeemed_amount {
            Some(redeemed_amount.to_guard_format()?)
        } else {
            None
        };
        // address gate
        let address_gate = if let Some(address_gate) = &self.address_gate {
            Some(address_gate.to_guard_format()?)
        } else {
            None
        };
        // nft gate
        let nft_gate = if let Some(nft_gate) = &self.nft_gate {
            Some(nft_gate.to_guard_format()?)
        } else {
            None
        };
        // nft burn
        let nft_burn = if let Some(nft_burn) = &self.nft_burn {
            Some(nft_burn.to_guard_format()?)
        } else {
            None
        };
        // token burn
        let token_burn = if let Some(token_burn) = &self.token_burn {
            Some(token_burn.to_guard_format()?)
        } else {
            None
        };
        // freeze sol payment
        let freeze_sol_payment = if let Some(freeze_sol_payment) = &self.freeze_sol_payment {
            Some(freeze_sol_payment.to_guard_format()?)
        } else {
            None
        };
        // freeze token payment
        let freeze_token_payment = if let Some(freeze_token_payment) = &self.freeze_token_payment {
            Some(freeze_token_payment.to_guard_format()?)
        } else {
            None
        };

        Ok(mpl_candy_guard::guards::GuardSet {
            bot_tax,
            sol_payment,
            token_payment,
            start_date,
            third_party_signer,
            token_gate,
            gatekeeper,
            end_date,
            allow_list,
            mint_limit,
            nft_payment,
            redeemed_amount,
            address_gate,
            nft_gate,
            nft_burn,
            token_burn,
            freeze_sol_payment,
            freeze_token_payment,
            program_gate: None,
            allocation: None,
        })
    }
}

// Address guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AddressGate {
    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub address: Pubkey,
}

impl AddressGate {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::AddressGate> {
        Ok(mpl_candy_guard::guards::AddressGate {
            address: self.address,
        })
    }
}

// Alow List guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AllowList {
    pub merkle_root: String,
}

impl AllowList {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::AllowList> {
        let root: [u8; 32] = hex::decode(&self.merkle_root)?
            .try_into()
            .map_err(|_| anyhow!("Invalid merkle root value: {}", self.merkle_root))?;
        Ok(mpl_candy_guard::guards::AllowList { merkle_root: root })
    }
}

// Bot Tax guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct BotTax {
    pub value: f64,

    pub last_instruction: bool,
}

impl BotTax {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::BotTax> {
        Ok(mpl_candy_guard::guards::BotTax {
            lamports: price_as_lamports(self.value),
            last_instruction: self.last_instruction,
        })
    }
}

// End Date guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EndDate {
    pub date: String,
}

impl EndDate {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::EndDate> {
        let timestamp = self.date.parse::<DateTimeUtc>()?.0.timestamp();

        Ok(mpl_candy_guard::guards::EndDate { date: timestamp })
    }
}

// Gatekeeper guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Gatekeeper {
    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub gatekeeper_network: Pubkey,

    pub expire_on_use: bool,
}

impl Gatekeeper {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::Gatekeeper> {
        Ok(mpl_candy_guard::guards::Gatekeeper {
            gatekeeper_network: self.gatekeeper_network,
            expire_on_use: self.expire_on_use,
        })
    }
}

// Mint Limit guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MintLimit {
    pub id: u8,

    pub limit: u16,
}

impl MintLimit {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::MintLimit> {
        Ok(mpl_candy_guard::guards::MintLimit {
            id: self.id,
            limit: self.limit,
        })
    }
}

// Nft Burn guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct NftBurn {
    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub required_collection: Pubkey,
}

impl NftBurn {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::NftBurn> {
        Ok(mpl_candy_guard::guards::NftBurn {
            required_collection: self.required_collection,
        })
    }
}

// Nft Gate guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct NftGate {
    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub required_collection: Pubkey,
}

impl NftGate {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::NftGate> {
        Ok(mpl_candy_guard::guards::NftGate {
            required_collection: self.required_collection,
        })
    }
}

// Nft Payment guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct NftPayment {
    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub required_collection: Pubkey,

    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub destination: Pubkey,
}

impl NftPayment {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::NftPayment> {
        Ok(mpl_candy_guard::guards::NftPayment {
            required_collection: self.required_collection,
            destination: self.destination,
        })
    }
}

// Redeemed Amount guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RedeemedAmount {
    pub maximum: u64,
}

impl RedeemedAmount {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::RedeemedAmount> {
        Ok(mpl_candy_guard::guards::RedeemedAmount {
            maximum: self.maximum,
        })
    }
}

// Sol Payment guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SolPayment {
    pub value: f64,

    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub destination: Pubkey,
}

impl SolPayment {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::SolPayment> {
        Ok(mpl_candy_guard::guards::SolPayment {
            lamports: price_as_lamports(self.value),
            destination: self.destination,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StartDate {
    pub date: String,
}

impl StartDate {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::StartDate> {
        let timestamp = self.date.parse::<DateTimeUtc>()?.0.timestamp();
        Ok(mpl_candy_guard::guards::StartDate { date: timestamp })
    }
}

// Third Party Signer guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ThirdPartySigner {
    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub signer_key: Pubkey,
}

impl ThirdPartySigner {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::ThirdPartySigner> {
        Ok(mpl_candy_guard::guards::ThirdPartySigner {
            signer_key: self.signer_key,
        })
    }
}

// Token Burn guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TokenBurn {
    pub amount: u64,

    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub mint: Pubkey,
}

impl TokenBurn {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::TokenBurn> {
        Ok(mpl_candy_guard::guards::TokenBurn {
            amount: self.amount,
            mint: self.mint,
        })
    }
}

// Token Gate guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TokenGate {
    pub amount: u64,

    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub mint: Pubkey,
}

impl TokenGate {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::TokenGate> {
        Ok(mpl_candy_guard::guards::TokenGate {
            amount: self.amount,
            mint: self.mint,
        })
    }
}

// Token Payment guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenPayment {
    pub amount: u64,

    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub mint: Pubkey,

    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub destination_ata: Pubkey,
}

impl TokenPayment {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::TokenPayment> {
        Ok(mpl_candy_guard::guards::TokenPayment {
            amount: self.amount,
            mint: self.mint,
            destination_ata: self.destination_ata,
        })
    }
}

// Freeze Sol Payment guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct FreezeSolPayment {
    pub value: f64,

    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub destination: Pubkey,
}

impl FreezeSolPayment {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::FreezeSolPayment> {
        Ok(mpl_candy_guard::guards::FreezeSolPayment {
            lamports: price_as_lamports(self.value),
            destination: self.destination,
        })
    }
}

// Freeze Token Payment guard

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct FreezeTokenPayment {
    pub amount: u64,

    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub mint: Pubkey,

    #[serde(deserialize_with = "to_pubkey")]
    #[serde(serialize_with = "to_string")]
    pub destination_ata: Pubkey,
}

impl FreezeTokenPayment {
    pub fn to_guard_format(&self) -> Result<mpl_candy_guard::guards::FreezeTokenPayment> {
        Ok(mpl_candy_guard::guards::FreezeTokenPayment {
            amount: self.amount,
            mint: self.mint,
            destination_ata: self.destination_ata,
        })
    }
}
