pub use target_arch::*;

#[cfg(not(target_arch = "bpf"))]
mod target_arch {
    use {
        crate::{encryption::elgamal::ElGamalCiphertext, zk_token_elgamal::pod},
        curve25519_dalek::{constants::RISTRETTO_BASEPOINT_COMPRESSED, scalar::Scalar},
        std::convert::TryInto,
    };
    pub const TWO_32: u64 = 4294967296;

    // On input two scalars x0, x1 and two ciphertexts ct0, ct1,
    // returns `Some(x0*ct0 + x1*ct1)` or `None` if the input was invalid
    fn add_ciphertexts(
        scalar_0: Scalar,
        ct_0: &pod::ElGamalCiphertext,
        scalar_1: Scalar,
        ct_1: &pod::ElGamalCiphertext,
    ) -> Option<pod::ElGamalCiphertext> {
        let ct_0: ElGamalCiphertext = (*ct_0).try_into().ok()?;
        let ct_1: ElGamalCiphertext = (*ct_1).try_into().ok()?;

        let ct_sum = ct_0 * scalar_0 + ct_1 * scalar_1;
        Some(pod::ElGamalCiphertext::from(ct_sum))
    }

    pub(crate) fn combine_lo_hi(
        ct_lo: &pod::ElGamalCiphertext,
        ct_hi: &pod::ElGamalCiphertext,
    ) -> Option<pod::ElGamalCiphertext> {
        add_ciphertexts(Scalar::one(), ct_lo, Scalar::from(TWO_32), ct_hi)
    }

    pub fn add(
        ct_0: &pod::ElGamalCiphertext,
        ct_1: &pod::ElGamalCiphertext,
    ) -> Option<pod::ElGamalCiphertext> {
        add_ciphertexts(Scalar::one(), ct_0, Scalar::one(), ct_1)
    }

    pub fn add_with_lo_hi(
        ct_0: &pod::ElGamalCiphertext,
        ct_1_lo: &pod::ElGamalCiphertext,
        ct_1_hi: &pod::ElGamalCiphertext,
    ) -> Option<pod::ElGamalCiphertext> {
        let ct_1 = combine_lo_hi(ct_1_lo, ct_1_hi)?;
        add_ciphertexts(Scalar::one(), ct_0, Scalar::one(), &ct_1)
    }

    pub fn subtract(
        ct_0: &pod::ElGamalCiphertext,
        ct_1: &pod::ElGamalCiphertext,
    ) -> Option<pod::ElGamalCiphertext> {
        add_ciphertexts(Scalar::one(), ct_0, -Scalar::one(), ct_1)
    }

    pub fn subtract_with_lo_hi(
        ct_0: &pod::ElGamalCiphertext,
        ct_1_lo: &pod::ElGamalCiphertext,
        ct_1_hi: &pod::ElGamalCiphertext,
    ) -> Option<pod::ElGamalCiphertext> {
        let ct_1 = combine_lo_hi(ct_1_lo, ct_1_hi)?;
        add_ciphertexts(Scalar::one(), ct_0, -Scalar::one(), &ct_1)
    }

    pub fn add_to(ct: &pod::ElGamalCiphertext, amount: u64) -> Option<pod::ElGamalCiphertext> {
        let mut amount_as_ct = [0_u8; 64];
        amount_as_ct[..32].copy_from_slice(RISTRETTO_BASEPOINT_COMPRESSED.as_bytes());
        add_ciphertexts(
            Scalar::one(),
            ct,
            Scalar::from(amount),
            &pod::ElGamalCiphertext(amount_as_ct),
        )
    }

    pub fn subtract_from(
        ct: &pod::ElGamalCiphertext,
        amount: u64,
    ) -> Option<pod::ElGamalCiphertext> {
        let mut amount_as_ct = [0_u8; 64];
        amount_as_ct[..32].copy_from_slice(RISTRETTO_BASEPOINT_COMPRESSED.as_bytes());
        add_ciphertexts(
            Scalar::one(),
            ct,
            -Scalar::from(amount),
            &pod::ElGamalCiphertext(amount_as_ct),
        )
    }
}

#[cfg(target_arch = "bpf")]
#[allow(unused_variables)]
mod target_arch {
    use {super::*, crate::zk_token_elgamal::pod, bytemuck::Zeroable};

    fn op(
        op: u64,
        ct_0: &pod::ElGamalCiphertext,
        ct_1: &pod::ElGamalCiphertext,
    ) -> Option<pod::ElGamalCiphertext> {
        let mut ct_result = pod::ElGamalCiphertext::zeroed();
        let result = unsafe {
            sol_zk_token_elgamal_op(
                op,
                &ct_0.0 as *const u8,
                &ct_1.0 as *const u8,
                &mut ct_result.0 as *mut u8,
            )
        };

        if result == 0 {
            Some(ct_result)
        } else {
            None
        }
    }

    fn op_with_lo_hi(
        op: u64,
        ct_0: &pod::ElGamalCiphertext,
        ct_1_lo: &pod::ElGamalCiphertext,
        ct_1_hi: &pod::ElGamalCiphertext,
    ) -> Option<pod::ElGamalCiphertext> {
        let mut ct_result = pod::ElGamalCiphertext::zeroed();
        let result = unsafe {
            sol_zk_token_elgamal_op_with_lo_hi(
                op,
                &ct_0.0 as *const u8,
                &ct_1_lo.0 as *const u8,
                &ct_1_hi.0 as *const u8,
                &mut ct_result.0 as *mut u8,
            )
        };

        if result == 0 {
            Some(ct_result)
        } else {
            None
        }
    }

    fn op_with_scalar(
        op: u64,
        ct: &pod::ElGamalCiphertext,
        scalar: u64,
    ) -> Option<pod::ElGamalCiphertext> {
        let mut ct_result = pod::ElGamalCiphertext::zeroed();
        let result = unsafe {
            sol_zk_token_elgamal_op_with_scalar(
                op,
                &ct.0 as *const u8,
                scalar,
                &mut ct_result.0 as *mut u8,
            )
        };

        if result == 0 {
            Some(ct_result)
        } else {
            None
        }
    }

    pub fn add(
        ct_0: &pod::ElGamalCiphertext,
        ct_1: &pod::ElGamalCiphertext,
    ) -> Option<pod::ElGamalCiphertext> {
        op(OP_ADD, ct_0, ct_1)
    }

    pub fn add_with_lo_hi(
        ct_0: &pod::ElGamalCiphertext,
        ct_1_lo: &pod::ElGamalCiphertext,
        ct_1_hi: &pod::ElGamalCiphertext,
    ) -> Option<pod::ElGamalCiphertext> {
        op_with_lo_hi(OP_ADD, ct_0, ct_1_lo, ct_1_hi)
    }

    pub fn subtract(
        ct_0: &pod::ElGamalCiphertext,
        ct_1: &pod::ElGamalCiphertext,
    ) -> Option<pod::ElGamalCiphertext> {
        op(OP_SUB, ct_0, ct_1)
    }

    pub fn subtract_with_lo_hi(
        ct_0: &pod::ElGamalCiphertext,
        ct_1_lo: &pod::ElGamalCiphertext,
        ct_1_hi: &pod::ElGamalCiphertext,
    ) -> Option<pod::ElGamalCiphertext> {
        op_with_lo_hi(OP_SUB, ct_0, ct_1_lo, ct_1_hi)
    }

    pub fn add_to(ct: &pod::ElGamalCiphertext, amount: u64) -> Option<pod::ElGamalCiphertext> {
        op_with_scalar(OP_ADD, ct, amount)
    }

    pub fn subtract_from(
        ct: &pod::ElGamalCiphertext,
        amount: u64,
    ) -> Option<pod::ElGamalCiphertext> {
        op_with_scalar(OP_SUB, ct, amount)
    }
}

pub const OP_ADD: u64 = 0;
pub const OP_SUB: u64 = 1;

extern "C" {
    pub fn sol_zk_token_elgamal_op(
        op: u64,
        ct_0: *const u8,
        ct_1: *const u8,
        ct_result: *mut u8,
    ) -> u64;
    pub fn sol_zk_token_elgamal_op_with_lo_hi(
        op: u64,
        ct_0: *const u8,
        ct_1_lo: *const u8,
        ct_1_hi: *const u8,
        ct_result: *mut u8,
    ) -> u64;
    pub fn sol_zk_token_elgamal_op_with_scalar(
        op: u64,
        ct: *const u8,
        scalar: u64,
        ct_result: *mut u8,
    ) -> u64;
}
