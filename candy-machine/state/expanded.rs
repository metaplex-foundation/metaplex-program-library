#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;
/// Candy machine state and config data.
pub struct CandyMachine {
    pub authority: Pubkey,
    pub wallet: Pubkey,
    pub token_mint: Option<Pubkey>,
    pub items_redeemed: u64,
    pub data: CandyMachineData,
}
impl borsh::ser::BorshSerialize for CandyMachine
where
    Pubkey: borsh::ser::BorshSerialize,
    Pubkey: borsh::ser::BorshSerialize,
    Option<Pubkey>: borsh::ser::BorshSerialize,
    u64: borsh::ser::BorshSerialize,
    CandyMachineData: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self.authority, writer)?;
        borsh::BorshSerialize::serialize(&self.wallet, writer)?;
        borsh::BorshSerialize::serialize(&self.token_mint, writer)?;
        borsh::BorshSerialize::serialize(&self.items_redeemed, writer)?;
        borsh::BorshSerialize::serialize(&self.data, writer)?;
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for CandyMachine
where
    Pubkey: borsh::BorshDeserialize,
    Pubkey: borsh::BorshDeserialize,
    Option<Pubkey>: borsh::BorshDeserialize,
    u64: borsh::BorshDeserialize,
    CandyMachineData: borsh::BorshDeserialize,
{
    fn deserialize(
        buf: &mut &[u8],
    ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            authority: borsh::BorshDeserialize::deserialize(buf)?,
            wallet: borsh::BorshDeserialize::deserialize(buf)?,
            token_mint: borsh::BorshDeserialize::deserialize(buf)?,
            items_redeemed: borsh::BorshDeserialize::deserialize(buf)?,
            data: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
#[automatically_derived]
impl ::core::clone::Clone for CandyMachine {
    #[inline]
    fn clone(&self) -> CandyMachine {
        CandyMachine {
            authority: ::core::clone::Clone::clone(&self.authority),
            wallet: ::core::clone::Clone::clone(&self.wallet),
            token_mint: ::core::clone::Clone::clone(&self.token_mint),
            items_redeemed: ::core::clone::Clone::clone(&self.items_redeemed),
            data: ::core::clone::Clone::clone(&self.data),
        }
    }
}
#[automatically_derived]
impl ::core::default::Default for CandyMachine {
    #[inline]
    fn default() -> CandyMachine {
        CandyMachine {
            authority: ::core::default::Default::default(),
            wallet: ::core::default::Default::default(),
            token_mint: ::core::default::Default::default(),
            items_redeemed: ::core::default::Default::default(),
            data: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for CandyMachine {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field5_finish(
            f,
            "CandyMachine",
            "authority",
            &&self.authority,
            "wallet",
            &&self.wallet,
            "token_mint",
            &&self.token_mint,
            "items_redeemed",
            &&self.items_redeemed,
            "data",
            &&self.data,
        )
    }
}
/// Candy machine settings data.
pub struct CandyMachineData {
    pub uuid: String,
    pub price: u64,
    /// The symbol for the asset
    pub symbol: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    pub max_supply: u64,
    pub is_mutable: bool,
    pub retain_authority: bool,
    pub go_live_date: Option<i64>,
    pub end_settings: Option<EndSettings>,
    pub creators: Vec<Creator>,
    pub hidden_settings: Option<HiddenSettings>,
    pub whitelist_mint_settings: Option<WhitelistMintSettings>,
    pub items_available: u64,
    /// If [`Some`] requires gateway tokens on mint
    pub gatekeeper: Option<GatekeeperConfig>,
}
impl borsh::ser::BorshSerialize for CandyMachineData
where
    String: borsh::ser::BorshSerialize,
    u64: borsh::ser::BorshSerialize,
    String: borsh::ser::BorshSerialize,
    u16: borsh::ser::BorshSerialize,
    u64: borsh::ser::BorshSerialize,
    bool: borsh::ser::BorshSerialize,
    bool: borsh::ser::BorshSerialize,
    Option<i64>: borsh::ser::BorshSerialize,
    Option<EndSettings>: borsh::ser::BorshSerialize,
    Vec<Creator>: borsh::ser::BorshSerialize,
    Option<HiddenSettings>: borsh::ser::BorshSerialize,
    Option<WhitelistMintSettings>: borsh::ser::BorshSerialize,
    u64: borsh::ser::BorshSerialize,
    Option<GatekeeperConfig>: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self.uuid, writer)?;
        borsh::BorshSerialize::serialize(&self.price, writer)?;
        borsh::BorshSerialize::serialize(&self.symbol, writer)?;
        borsh::BorshSerialize::serialize(&self.seller_fee_basis_points, writer)?;
        borsh::BorshSerialize::serialize(&self.max_supply, writer)?;
        borsh::BorshSerialize::serialize(&self.is_mutable, writer)?;
        borsh::BorshSerialize::serialize(&self.retain_authority, writer)?;
        borsh::BorshSerialize::serialize(&self.go_live_date, writer)?;
        borsh::BorshSerialize::serialize(&self.end_settings, writer)?;
        borsh::BorshSerialize::serialize(&self.creators, writer)?;
        borsh::BorshSerialize::serialize(&self.hidden_settings, writer)?;
        borsh::BorshSerialize::serialize(&self.whitelist_mint_settings, writer)?;
        borsh::BorshSerialize::serialize(&self.items_available, writer)?;
        borsh::BorshSerialize::serialize(&self.gatekeeper, writer)?;
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for CandyMachineData
where
    String: borsh::BorshDeserialize,
    u64: borsh::BorshDeserialize,
    String: borsh::BorshDeserialize,
    u16: borsh::BorshDeserialize,
    u64: borsh::BorshDeserialize,
    bool: borsh::BorshDeserialize,
    bool: borsh::BorshDeserialize,
    Option<i64>: borsh::BorshDeserialize,
    Option<EndSettings>: borsh::BorshDeserialize,
    Vec<Creator>: borsh::BorshDeserialize,
    Option<HiddenSettings>: borsh::BorshDeserialize,
    Option<WhitelistMintSettings>: borsh::BorshDeserialize,
    u64: borsh::BorshDeserialize,
    Option<GatekeeperConfig>: borsh::BorshDeserialize,
{
    fn deserialize(
        buf: &mut &[u8],
    ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            uuid: borsh::BorshDeserialize::deserialize(buf)?,
            price: borsh::BorshDeserialize::deserialize(buf)?,
            symbol: borsh::BorshDeserialize::deserialize(buf)?,
            seller_fee_basis_points: borsh::BorshDeserialize::deserialize(buf)?,
            max_supply: borsh::BorshDeserialize::deserialize(buf)?,
            is_mutable: borsh::BorshDeserialize::deserialize(buf)?,
            retain_authority: borsh::BorshDeserialize::deserialize(buf)?,
            go_live_date: borsh::BorshDeserialize::deserialize(buf)?,
            end_settings: borsh::BorshDeserialize::deserialize(buf)?,
            creators: borsh::BorshDeserialize::deserialize(buf)?,
            hidden_settings: borsh::BorshDeserialize::deserialize(buf)?,
            whitelist_mint_settings: borsh::BorshDeserialize::deserialize(buf)?,
            items_available: borsh::BorshDeserialize::deserialize(buf)?,
            gatekeeper: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
#[automatically_derived]
impl ::core::clone::Clone for CandyMachineData {
    #[inline]
    fn clone(&self) -> CandyMachineData {
        CandyMachineData {
            uuid: ::core::clone::Clone::clone(&self.uuid),
            price: ::core::clone::Clone::clone(&self.price),
            symbol: ::core::clone::Clone::clone(&self.symbol),
            seller_fee_basis_points: ::core::clone::Clone::clone(
                &self.seller_fee_basis_points,
            ),
            max_supply: ::core::clone::Clone::clone(&self.max_supply),
            is_mutable: ::core::clone::Clone::clone(&self.is_mutable),
            retain_authority: ::core::clone::Clone::clone(&self.retain_authority),
            go_live_date: ::core::clone::Clone::clone(&self.go_live_date),
            end_settings: ::core::clone::Clone::clone(&self.end_settings),
            creators: ::core::clone::Clone::clone(&self.creators),
            hidden_settings: ::core::clone::Clone::clone(&self.hidden_settings),
            whitelist_mint_settings: ::core::clone::Clone::clone(
                &self.whitelist_mint_settings,
            ),
            items_available: ::core::clone::Clone::clone(&self.items_available),
            gatekeeper: ::core::clone::Clone::clone(&self.gatekeeper),
        }
    }
}
#[automatically_derived]
impl ::core::default::Default for CandyMachineData {
    #[inline]
    fn default() -> CandyMachineData {
        CandyMachineData {
            uuid: ::core::default::Default::default(),
            price: ::core::default::Default::default(),
            symbol: ::core::default::Default::default(),
            seller_fee_basis_points: ::core::default::Default::default(),
            max_supply: ::core::default::Default::default(),
            is_mutable: ::core::default::Default::default(),
            retain_authority: ::core::default::Default::default(),
            go_live_date: ::core::default::Default::default(),
            end_settings: ::core::default::Default::default(),
            creators: ::core::default::Default::default(),
            hidden_settings: ::core::default::Default::default(),
            whitelist_mint_settings: ::core::default::Default::default(),
            items_available: ::core::default::Default::default(),
            gatekeeper: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for CandyMachineData {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "uuid",
            "price",
            "symbol",
            "seller_fee_basis_points",
            "max_supply",
            "is_mutable",
            "retain_authority",
            "go_live_date",
            "end_settings",
            "creators",
            "hidden_settings",
            "whitelist_mint_settings",
            "items_available",
            "gatekeeper",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &&self.uuid,
            &&self.price,
            &&self.symbol,
            &&self.seller_fee_basis_points,
            &&self.max_supply,
            &&self.is_mutable,
            &&self.retain_authority,
            &&self.go_live_date,
            &&self.end_settings,
            &&self.creators,
            &&self.hidden_settings,
            &&self.whitelist_mint_settings,
            &&self.items_available,
            &&self.gatekeeper,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(
            f,
            "CandyMachineData",
            names,
            values,
        )
    }
}
/// Collection PDA account
pub struct CollectionPDA {
    pub mint: Pubkey,
    pub candy_machine: Pubkey,
}
impl borsh::ser::BorshSerialize for CollectionPDA
where
    Pubkey: borsh::ser::BorshSerialize,
    Pubkey: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self.mint, writer)?;
        borsh::BorshSerialize::serialize(&self.candy_machine, writer)?;
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for CollectionPDA
where
    Pubkey: borsh::BorshDeserialize,
    Pubkey: borsh::BorshDeserialize,
{
    fn deserialize(
        buf: &mut &[u8],
    ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            mint: borsh::BorshDeserialize::deserialize(buf)?,
            candy_machine: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
#[automatically_derived]
impl ::core::clone::Clone for CollectionPDA {
    #[inline]
    fn clone(&self) -> CollectionPDA {
        CollectionPDA {
            mint: ::core::clone::Clone::clone(&self.mint),
            candy_machine: ::core::clone::Clone::clone(&self.candy_machine),
        }
    }
}
#[automatically_derived]
impl ::core::default::Default for CollectionPDA {
    #[inline]
    fn default() -> CollectionPDA {
        CollectionPDA {
            mint: ::core::default::Default::default(),
            candy_machine: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for CollectionPDA {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "CollectionPDA",
            "mint",
            &&self.mint,
            "candy_machine",
            &&self.candy_machine,
        )
    }
}
pub struct FreezePDA {
    pub candy_machine: Pubkey,
    pub allow_thaw: bool,
    pub frozen_count: u64,
    pub mint_start: Option<i64>,
    pub freeze_time: i64,
    pub freeze_fee: u64,
}
impl borsh::de::BorshDeserialize for FreezePDA
where
    Pubkey: borsh::BorshDeserialize,
    bool: borsh::BorshDeserialize,
    u64: borsh::BorshDeserialize,
    Option<i64>: borsh::BorshDeserialize,
    i64: borsh::BorshDeserialize,
    u64: borsh::BorshDeserialize,
{
    fn deserialize(
        buf: &mut &[u8],
    ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            candy_machine: borsh::BorshDeserialize::deserialize(buf)?,
            allow_thaw: borsh::BorshDeserialize::deserialize(buf)?,
            frozen_count: borsh::BorshDeserialize::deserialize(buf)?,
            mint_start: borsh::BorshDeserialize::deserialize(buf)?,
            freeze_time: borsh::BorshDeserialize::deserialize(buf)?,
            freeze_fee: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
impl borsh::ser::BorshSerialize for FreezePDA
where
    Pubkey: borsh::ser::BorshSerialize,
    bool: borsh::ser::BorshSerialize,
    u64: borsh::ser::BorshSerialize,
    Option<i64>: borsh::ser::BorshSerialize,
    i64: borsh::ser::BorshSerialize,
    u64: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self.candy_machine, writer)?;
        borsh::BorshSerialize::serialize(&self.allow_thaw, writer)?;
        borsh::BorshSerialize::serialize(&self.frozen_count, writer)?;
        borsh::BorshSerialize::serialize(&self.mint_start, writer)?;
        borsh::BorshSerialize::serialize(&self.freeze_time, writer)?;
        borsh::BorshSerialize::serialize(&self.freeze_fee, writer)?;
        Ok(())
    }
}
#[automatically_derived]
impl ::core::clone::Clone for FreezePDA {
    #[inline]
    fn clone(&self) -> FreezePDA {
        FreezePDA {
            candy_machine: ::core::clone::Clone::clone(&self.candy_machine),
            allow_thaw: ::core::clone::Clone::clone(&self.allow_thaw),
            frozen_count: ::core::clone::Clone::clone(&self.frozen_count),
            mint_start: ::core::clone::Clone::clone(&self.mint_start),
            freeze_time: ::core::clone::Clone::clone(&self.freeze_time),
            freeze_fee: ::core::clone::Clone::clone(&self.freeze_fee),
        }
    }
}
#[automatically_derived]
impl ::core::default::Default for FreezePDA {
    #[inline]
    fn default() -> FreezePDA {
        FreezePDA {
            candy_machine: ::core::default::Default::default(),
            allow_thaw: ::core::default::Default::default(),
            frozen_count: ::core::default::Default::default(),
            mint_start: ::core::default::Default::default(),
            freeze_time: ::core::default::Default::default(),
            freeze_fee: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for FreezePDA {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "candy_machine",
            "allow_thaw",
            "frozen_count",
            "mint_start",
            "freeze_time",
            "freeze_fee",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &&self.candy_machine,
            &&self.allow_thaw,
            &&self.frozen_count,
            &&self.mint_start,
            &&self.freeze_time,
            &&self.freeze_fee,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(f, "FreezePDA", names, values)
    }
}
impl ::core::marker::StructuralPartialEq for FreezePDA {}
#[automatically_derived]
impl ::core::cmp::PartialEq for FreezePDA {
    #[inline]
    fn eq(&self, other: &FreezePDA) -> bool {
        self.candy_machine == other.candy_machine && self.allow_thaw == other.allow_thaw
            && self.frozen_count == other.frozen_count
            && self.mint_start == other.mint_start
            && self.freeze_time == other.freeze_time
            && self.freeze_fee == other.freeze_fee
    }
}
impl ::core::marker::StructuralEq for FreezePDA {}
#[automatically_derived]
impl ::core::cmp::Eq for FreezePDA {
    #[inline]
    #[doc(hidden)]
    #[no_coverage]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<Pubkey>;
        let _: ::core::cmp::AssertParamIsEq<bool>;
        let _: ::core::cmp::AssertParamIsEq<u64>;
        let _: ::core::cmp::AssertParamIsEq<Option<i64>>;
        let _: ::core::cmp::AssertParamIsEq<i64>;
    }
}
/// Individual config line for storing NFT data pre-mint.
pub struct ConfigLine {
    pub name: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
}
impl borsh::ser::BorshSerialize for ConfigLine
where
    String: borsh::ser::BorshSerialize,
    String: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self.name, writer)?;
        borsh::BorshSerialize::serialize(&self.uri, writer)?;
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for ConfigLine
where
    String: borsh::BorshDeserialize,
    String: borsh::BorshDeserialize,
{
    fn deserialize(
        buf: &mut &[u8],
    ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            name: borsh::BorshDeserialize::deserialize(buf)?,
            uri: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for ConfigLine {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "ConfigLine",
            "name",
            &&self.name,
            "uri",
            &&self.uri,
        )
    }
}
pub struct EndSettings {
    pub end_setting_type: EndSettingType,
    pub number: u64,
}
impl borsh::ser::BorshSerialize for EndSettings
where
    EndSettingType: borsh::ser::BorshSerialize,
    u64: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self.end_setting_type, writer)?;
        borsh::BorshSerialize::serialize(&self.number, writer)?;
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for EndSettings
where
    EndSettingType: borsh::BorshDeserialize,
    u64: borsh::BorshDeserialize,
{
    fn deserialize(
        buf: &mut &[u8],
    ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            end_setting_type: borsh::BorshDeserialize::deserialize(buf)?,
            number: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
#[automatically_derived]
impl ::core::clone::Clone for EndSettings {
    #[inline]
    fn clone(&self) -> EndSettings {
        EndSettings {
            end_setting_type: ::core::clone::Clone::clone(&self.end_setting_type),
            number: ::core::clone::Clone::clone(&self.number),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for EndSettings {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "EndSettings",
            "end_setting_type",
            &&self.end_setting_type,
            "number",
            &&self.number,
        )
    }
}
pub enum EndSettingType {
    Date,
    Amount,
}
impl borsh::ser::BorshSerialize for EndSettingType {
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> core::result::Result<(), borsh::maybestd::io::Error> {
        let variant_idx: u8 = match self {
            EndSettingType::Date => 0u8,
            EndSettingType::Amount => 1u8,
        };
        writer.write_all(&variant_idx.to_le_bytes())?;
        match self {
            EndSettingType::Date => {}
            EndSettingType::Amount => {}
        }
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for EndSettingType {
    fn deserialize(
        buf: &mut &[u8],
    ) -> core::result::Result<Self, borsh::maybestd::io::Error> {
        let variant_idx: u8 = borsh::BorshDeserialize::deserialize(buf)?;
        let return_value = match variant_idx {
            0u8 => EndSettingType::Date,
            1u8 => EndSettingType::Amount,
            _ => {
                let msg = {
                    let res = ::alloc::fmt::format(
                        ::core::fmt::Arguments::new_v1(
                            &["Unexpected variant index: "],
                            &[::core::fmt::ArgumentV1::new_debug(&variant_idx)],
                        ),
                    );
                    res
                };
                return Err(
                    borsh::maybestd::io::Error::new(
                        borsh::maybestd::io::ErrorKind::InvalidInput,
                        msg,
                    ),
                );
            }
        };
        Ok(return_value)
    }
}
#[automatically_derived]
impl ::core::clone::Clone for EndSettingType {
    #[inline]
    fn clone(&self) -> EndSettingType {
        match self {
            EndSettingType::Date => EndSettingType::Date,
            EndSettingType::Amount => EndSettingType::Amount,
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for EndSettingType {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            EndSettingType::Date => ::core::fmt::Formatter::write_str(f, "Date"),
            EndSettingType::Amount => ::core::fmt::Formatter::write_str(f, "Amount"),
        }
    }
}
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    pub share: u8,
}
impl borsh::ser::BorshSerialize for Creator
where
    Pubkey: borsh::ser::BorshSerialize,
    bool: borsh::ser::BorshSerialize,
    u8: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self.address, writer)?;
        borsh::BorshSerialize::serialize(&self.verified, writer)?;
        borsh::BorshSerialize::serialize(&self.share, writer)?;
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for Creator
where
    Pubkey: borsh::BorshDeserialize,
    bool: borsh::BorshDeserialize,
    u8: borsh::BorshDeserialize,
{
    fn deserialize(
        buf: &mut &[u8],
    ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            address: borsh::BorshDeserialize::deserialize(buf)?,
            verified: borsh::BorshDeserialize::deserialize(buf)?,
            share: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Creator {
    #[inline]
    fn clone(&self) -> Creator {
        Creator {
            address: ::core::clone::Clone::clone(&self.address),
            verified: ::core::clone::Clone::clone(&self.verified),
            share: ::core::clone::Clone::clone(&self.share),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Creator {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "Creator",
            "address",
            &&self.address,
            "verified",
            &&self.verified,
            "share",
            &&self.share,
        )
    }
}
/// Hidden Settings for large mints used with offline data.
pub struct HiddenSettings {
    pub name: String,
    pub uri: String,
    pub hash: [u8; 32],
}
impl borsh::ser::BorshSerialize for HiddenSettings
where
    String: borsh::ser::BorshSerialize,
    String: borsh::ser::BorshSerialize,
    [u8; 32]: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self.name, writer)?;
        borsh::BorshSerialize::serialize(&self.uri, writer)?;
        borsh::BorshSerialize::serialize(&self.hash, writer)?;
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for HiddenSettings
where
    String: borsh::BorshDeserialize,
    String: borsh::BorshDeserialize,
    [u8; 32]: borsh::BorshDeserialize,
{
    fn deserialize(
        buf: &mut &[u8],
    ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            name: borsh::BorshDeserialize::deserialize(buf)?,
            uri: borsh::BorshDeserialize::deserialize(buf)?,
            hash: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
#[automatically_derived]
impl ::core::clone::Clone for HiddenSettings {
    #[inline]
    fn clone(&self) -> HiddenSettings {
        HiddenSettings {
            name: ::core::clone::Clone::clone(&self.name),
            uri: ::core::clone::Clone::clone(&self.uri),
            hash: ::core::clone::Clone::clone(&self.hash),
        }
    }
}
#[automatically_derived]
impl ::core::default::Default for HiddenSettings {
    #[inline]
    fn default() -> HiddenSettings {
        HiddenSettings {
            name: ::core::default::Default::default(),
            uri: ::core::default::Default::default(),
            hash: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for HiddenSettings {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "HiddenSettings",
            "name",
            &&self.name,
            "uri",
            &&self.uri,
            "hash",
            &&self.hash,
        )
    }
}
pub struct WhitelistMintSettings {
    pub mode: WhitelistMintMode,
    pub mint: Pubkey,
    pub presale: bool,
    pub discount_price: Option<u64>,
}
impl borsh::ser::BorshSerialize for WhitelistMintSettings
where
    WhitelistMintMode: borsh::ser::BorshSerialize,
    Pubkey: borsh::ser::BorshSerialize,
    bool: borsh::ser::BorshSerialize,
    Option<u64>: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self.mode, writer)?;
        borsh::BorshSerialize::serialize(&self.mint, writer)?;
        borsh::BorshSerialize::serialize(&self.presale, writer)?;
        borsh::BorshSerialize::serialize(&self.discount_price, writer)?;
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for WhitelistMintSettings
where
    WhitelistMintMode: borsh::BorshDeserialize,
    Pubkey: borsh::BorshDeserialize,
    bool: borsh::BorshDeserialize,
    Option<u64>: borsh::BorshDeserialize,
{
    fn deserialize(
        buf: &mut &[u8],
    ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            mode: borsh::BorshDeserialize::deserialize(buf)?,
            mint: borsh::BorshDeserialize::deserialize(buf)?,
            presale: borsh::BorshDeserialize::deserialize(buf)?,
            discount_price: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
#[automatically_derived]
impl ::core::clone::Clone for WhitelistMintSettings {
    #[inline]
    fn clone(&self) -> WhitelistMintSettings {
        WhitelistMintSettings {
            mode: ::core::clone::Clone::clone(&self.mode),
            mint: ::core::clone::Clone::clone(&self.mint),
            presale: ::core::clone::Clone::clone(&self.presale),
            discount_price: ::core::clone::Clone::clone(&self.discount_price),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for WhitelistMintSettings {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field4_finish(
            f,
            "WhitelistMintSettings",
            "mode",
            &&self.mode,
            "mint",
            &&self.mint,
            "presale",
            &&self.presale,
            "discount_price",
            &&self.discount_price,
        )
    }
}
pub enum WhitelistMintMode {
    BurnEveryTime,
    NeverBurn,
}
impl borsh::ser::BorshSerialize for WhitelistMintMode {
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> core::result::Result<(), borsh::maybestd::io::Error> {
        let variant_idx: u8 = match self {
            WhitelistMintMode::BurnEveryTime => 0u8,
            WhitelistMintMode::NeverBurn => 1u8,
        };
        writer.write_all(&variant_idx.to_le_bytes())?;
        match self {
            WhitelistMintMode::BurnEveryTime => {}
            WhitelistMintMode::NeverBurn => {}
        }
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for WhitelistMintMode {
    fn deserialize(
        buf: &mut &[u8],
    ) -> core::result::Result<Self, borsh::maybestd::io::Error> {
        let variant_idx: u8 = borsh::BorshDeserialize::deserialize(buf)?;
        let return_value = match variant_idx {
            0u8 => WhitelistMintMode::BurnEveryTime,
            1u8 => WhitelistMintMode::NeverBurn,
            _ => {
                let msg = {
                    let res = ::alloc::fmt::format(
                        ::core::fmt::Arguments::new_v1(
                            &["Unexpected variant index: "],
                            &[::core::fmt::ArgumentV1::new_debug(&variant_idx)],
                        ),
                    );
                    res
                };
                return Err(
                    borsh::maybestd::io::Error::new(
                        borsh::maybestd::io::ErrorKind::InvalidInput,
                        msg,
                    ),
                );
            }
        };
        Ok(return_value)
    }
}
#[automatically_derived]
impl ::core::clone::Clone for WhitelistMintMode {
    #[inline]
    fn clone(&self) -> WhitelistMintMode {
        match self {
            WhitelistMintMode::BurnEveryTime => WhitelistMintMode::BurnEveryTime,
            WhitelistMintMode::NeverBurn => WhitelistMintMode::NeverBurn,
        }
    }
}
impl ::core::marker::StructuralEq for WhitelistMintMode {}
#[automatically_derived]
impl ::core::cmp::Eq for WhitelistMintMode {
    #[inline]
    #[doc(hidden)]
    #[no_coverage]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
impl ::core::marker::StructuralPartialEq for WhitelistMintMode {}
#[automatically_derived]
impl ::core::cmp::PartialEq for WhitelistMintMode {
    #[inline]
    fn eq(&self, other: &WhitelistMintMode) -> bool {
        let __self_tag = ::core::intrinsics::discriminant_value(self);
        let __arg1_tag = ::core::intrinsics::discriminant_value(other);
        __self_tag == __arg1_tag
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for WhitelistMintMode {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            WhitelistMintMode::BurnEveryTime => {
                ::core::fmt::Formatter::write_str(f, "BurnEveryTime")
            }
            WhitelistMintMode::NeverBurn => {
                ::core::fmt::Formatter::write_str(f, "NeverBurn")
            }
        }
    }
}
/// Configurations options for the gatekeeper.
pub struct GatekeeperConfig {
    /// The network for the gateway token required
    pub gatekeeper_network: Pubkey,
    /// Whether or not the token should expire after minting.
    /// The gatekeeper network must support this if true.
    pub expire_on_use: bool,
}
impl borsh::ser::BorshSerialize for GatekeeperConfig
where
    Pubkey: borsh::ser::BorshSerialize,
    bool: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self.gatekeeper_network, writer)?;
        borsh::BorshSerialize::serialize(&self.expire_on_use, writer)?;
        Ok(())
    }
}
impl borsh::de::BorshDeserialize for GatekeeperConfig
where
    Pubkey: borsh::BorshDeserialize,
    bool: borsh::BorshDeserialize,
{
    fn deserialize(
        buf: &mut &[u8],
    ) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            gatekeeper_network: borsh::BorshDeserialize::deserialize(buf)?,
            expire_on_use: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
#[automatically_derived]
impl ::core::clone::Clone for GatekeeperConfig {
    #[inline]
    fn clone(&self) -> GatekeeperConfig {
        GatekeeperConfig {
            gatekeeper_network: ::core::clone::Clone::clone(&self.gatekeeper_network),
            expire_on_use: ::core::clone::Clone::clone(&self.expire_on_use),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for GatekeeperConfig {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "GatekeeperConfig",
            "gatekeeper_network",
            &&self.gatekeeper_network,
            "expire_on_use",
            &&self.expire_on_use,
        )
    }
}
