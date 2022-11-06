use modular_bitfield::{bitfield, specifiers::B12};

#[bitfield(bits = 16)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferEffects {
    pub track: bool,
    pub burn: bool,
    pub freeze: bool,
    pub freeze_parent: bool,
    pub empty_bytes: B12,
}

impl Default for TransferEffects {
    fn default() -> Self {
        TransferEffects::new().with_track(true)
    }
}

impl From<u16> for TransferEffects {
    fn from(num: u16) -> Self {
        TransferEffects::from_bytes(num.to_le_bytes())
    }
}

impl From<TransferEffects> for u16 {
    fn from(transfer_effects: TransferEffects) -> Self {
        u16::from_le_bytes(transfer_effects.into_bytes())
    }
}
