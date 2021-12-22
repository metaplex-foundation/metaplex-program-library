//! Solana program utilities for Plain Old Data types
use {
    bytemuck::{Pod, Zeroable},
    solana_program::{
        account_info::AccountInfo, instruction::Instruction, program_error::ProgramError,
        pubkey::Pubkey,
    },
    std::{
        cell::{Ref, RefMut},
        marker::PhantomData,
        ops::{Deref, DerefMut},
    },
};

/// The standard `bool` is not a `Pod`, define a replacement that is
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
#[repr(transparent)]
pub struct PodBool(u8);
impl From<bool> for PodBool {
    fn from(b: bool) -> Self {
        Self(if b { 1 } else { 0 })
    }
}
impl From<&PodBool> for bool {
    fn from(b: &PodBool) -> Self {
        b.0 != 0
    }
}

/// The standard `u64` can cause alignment issues when placed in a `Pod`, define a replacement that
/// is usable in all `Pod`s
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
#[repr(transparent)]
pub struct PodU64([u8; 8]);
impl From<u64> for PodU64 {
    fn from(n: u64) -> Self {
        Self(n.to_le_bytes())
    }
}
impl From<PodU64> for u64 {
    fn from(pod: PodU64) -> Self {
        Self::from_le_bytes(pod.0)
    }
}

/// On-chain size of a `Pod` type
pub fn pod_get_packed_len<T: Pod>() -> usize {
    std::mem::size_of::<T>()
}

/// Convert `Instruction` data into a `Pod` (zero copy)
pub fn pod_from_instruction_data<'a, T: Pod>(
    instruction: &'a Instruction,
    program_id: &Pubkey,
) -> Result<&'a T, ProgramError> {
    if instruction.program_id != *program_id {
        Err(ProgramError::InvalidArgument)
    } else {
        pod_from_bytes(&instruction.data).ok_or(ProgramError::InvalidArgument)
    }
}

/// Convert a `Pod` into a slice (zero copy)
pub fn pod_bytes_of<T: Pod>(t: &T) -> &[u8] {
    bytemuck::bytes_of(t)
}

/// Convert a slice into a `Pod` (zero copy)
pub fn pod_from_bytes<T: Pod>(bytes: &[u8]) -> Option<&T> {
    bytemuck::try_from_bytes(bytes).ok()
}

/// Maybe convert a slice into a `Pod` (zero copy)
///
/// Returns `None` if the slice is empty, but `Err` if all other lengths but `get_packed_len()`
/// This function exists primary because `Option<T>` is not a `Pod`.
pub fn pod_maybe_from_bytes<T: Pod>(bytes: &[u8]) -> Result<Option<&T>, ProgramError> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        bytemuck::try_from_bytes(bytes)
            .map(Some)
            .map_err(|_| ProgramError::InvalidArgument)
    }
}

/// Convert a slice into a mutable `Pod` (zero copy)
pub fn pod_from_bytes_mut<T: Pod>(bytes: &mut [u8]) -> Result<&mut T, ProgramError> {
    bytemuck::try_from_bytes_mut(bytes).map_err(|_| ProgramError::InvalidArgument)
}

/// Represents a `Pod` within `AccountInfo::data`
pub struct PodAccountInfoData<'a, 'b, T: Pod> {
    account_info: &'a AccountInfo<'b>,
    account_data: Ref<'a, &'b mut [u8]>,
    phantom: PhantomData<&'a T>,
}

impl<'a, 'b, T: Pod> Deref for PodAccountInfoData<'a, 'b, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        pod_from_bytes(&self.account_data).unwrap()
    }
}

impl<'a, 'b, T: Pod> PodAccountInfoData<'a, 'b, T> {
    pub fn into_mut(self) -> PodAccountInfoDataMut<'a, 'b, T> {
        let account_info = self.account_info;
        drop(self);

        let account_data = account_info.data.borrow_mut();
        PodAccountInfoDataMut {
            account_data,
            phantom: PhantomData::default(),
        }
    }
}

/// Utility trait to add a `from_account_info()` function to any `Pod` struct
pub trait PodAccountInfo<'a, 'b>: bytemuck::Pod {
    fn from_bytes(bytes: &[u8]) -> Option<&Self> {
        pod_from_bytes::<Self>(bytes)
    }

    fn from_account_info(
        account_info: &'a AccountInfo<'b>,
        owner: &Pubkey,
    ) -> Result<PodAccountInfoData<'a, 'b, Self>, ProgramError> {
        if account_info.owner != owner {
            return Err(ProgramError::InvalidArgument);
        }

        let account_data = account_info.data.borrow();
        let _ = Self::from_bytes(&account_data).ok_or(ProgramError::InvalidArgument)?;
        Ok(PodAccountInfoData {
            account_info,
            account_data,
            phantom: PhantomData::default(),
        })
    }

    /// Get the packed length
    fn get_packed_len() -> usize {
        pod_get_packed_len::<Self>()
    }
}

/// Represents a mutable `Pod` within `AccountInfo::data`
pub struct PodAccountInfoDataMut<'a, 'b, T: Pod> {
    account_data: RefMut<'a, &'b mut [u8]>,
    phantom: PhantomData<&'a T>,
}

impl<'a, 'b, T: Pod> Deref for PodAccountInfoDataMut<'a, 'b, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        pod_from_bytes(&self.account_data).unwrap()
    }
}

impl<'a, 'b, T: Pod> DerefMut for PodAccountInfoDataMut<'a, 'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        pod_from_bytes_mut(&mut self.account_data).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pod_bool() {
        assert!(pod_from_bytes::<PodBool>(&[]).is_none());
        assert!(pod_from_bytes::<PodBool>(&[0, 0]).is_none());

        for i in 0..=u8::MAX {
            assert_eq!(i != 0, bool::from(pod_from_bytes::<PodBool>(&[i]).unwrap()));
        }
    }

    #[test]
    fn test_pod_u64() {
        assert!(pod_from_bytes::<PodU64>(&[]).is_none());
        assert_eq!(
            1u64,
            u64::from(*pod_from_bytes::<PodU64>(&[1, 0, 0, 0, 0, 0, 0, 0]).unwrap())
        );
    }
}
