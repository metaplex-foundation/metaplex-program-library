use std::ops::{Deref, DerefMut};

use crate::constants::FREEZE_FEE;
use crate::CandyError;
use anchor_lang::prelude::*;

use crate::*;

/// Candy machine state and config data.
#[account]
#[derive(Default, Debug)]
pub struct CandyMachine(pub CandyMachineState);

impl CandyMachine {
    pub fn new(state: CandyMachineState) -> Self {
        Self(state)
    }
    pub fn assert_not_minted(&self, candy_error: Error) -> Result<()> {
        if self.items_redeemed > 0 {
            Err(candy_error)
        } else {
            Ok(())
        }
    }
}

impl Deref for CandyMachine {
    type Target = CandyMachineState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CandyMachine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Collection PDA account
#[account]
#[derive(Default, Debug)]
pub struct CollectionPDA(pub CollectionPDAState);

impl CollectionPDA {
    pub const PREFIX: &'static str = "collection";
}

impl Deref for CollectionPDA {
    type Target = CollectionPDAState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CollectionPDA {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Collection PDA account
#[account]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct FreezePDA(pub FreezePDAState);

impl FreezePDA {
    pub fn new(state: FreezePDAState) -> Self {
        Self(state)
    }
}

impl Deref for FreezePDA {
    type Target = FreezePDAState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FreezePDA {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FreezePDA {
    pub const SIZE: usize = 8 + 32 + 32 + 1 + 8 + 1 + 8 + 8 + 8;

    pub const PREFIX: &'static str = "freeze";

    pub fn init(&mut self, candy_machine: Pubkey, mint_start: Option<i64>, freeze_time: i64) {
        self.0.candy_machine = candy_machine;
        self.0.allow_thaw = false;
        self.0.frozen_count = 0;
        self.0.mint_start = mint_start;
        self.0.freeze_time = freeze_time;
        self.0.freeze_fee = FREEZE_FEE;
    }

    pub fn thaw_eligible(&self, current_timestamp: i64, candy_machine: &CandyMachine) -> bool {
        if self.allow_thaw || candy_machine.items_redeemed >= candy_machine.data.items_available {
            return true;
        } else if let Some(start_timestamp) = self.mint_start {
            if current_timestamp >= start_timestamp + self.freeze_time {
                return true;
            }
        }
        false
    }

    pub fn assert_from_candy(&self, candy_machine: &Pubkey) -> Result<()> {
        if &self.candy_machine != candy_machine {
            return err!(CandyError::FreezePDAMismatch);
        }
        Ok(())
    }
}

/// Candy machine settings data.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
pub struct CandyMachineData(pub CandyMachineDataState);

impl CandyMachineData {
    pub fn new(state: CandyMachineDataState) -> Self {
        Self(state)
    }
}

impl Deref for CandyMachineData {
    type Target = CandyMachineDataState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CandyMachineData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Individual config line for storing NFT data pre-mint.
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct ConfigLine(pub ConfigLineState);

impl ConfigLine {
    pub fn new(state: ConfigLineState) -> Self {
        Self(state)
    }
}

impl Deref for ConfigLine {
    type Target = ConfigLineState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EndSettings(pub EndSettingsState);

impl Deref for EndSettings {
    type Target = EndSettingsState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Unfortunate duplication of token metadata so that IDL picks it up.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Creator(pub CreatorState);

impl Creator {
    pub fn new(state: CreatorState) -> Self {
        Self(state)
    }
}

impl Deref for Creator {
    type Target = CreatorState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
/// Hidden Settings for large mints used with offline data.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Debug)]
pub struct HiddenSettings(pub HiddenSettingsState);

impl HiddenSettings {
    pub fn new(state: HiddenSettingsState) -> Self {
        Self(state)
    }
}

impl Deref for HiddenSettings {
    type Target = HiddenSettingsState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct WhitelistMintSettings(pub WhitelistMintSettingsState);

impl WhitelistMintSettings {
    pub fn new(state: WhitelistMintSettingsState) -> Self {
        Self(state)
    }
}

impl Deref for WhitelistMintSettings {
    type Target = WhitelistMintSettingsState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Configurations options for the gatekeeper.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GatekeeperConfig(pub GatekeeperConfigState);

impl GatekeeperConfig {
    pub fn new(state: GatekeeperConfigState) -> Self {
        Self(state)
    }
}

impl Deref for GatekeeperConfig {
    type Target = GatekeeperConfigState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
