#![allow(non_snake_case)]
use {
    crate::encryption::{
        pedersen::{
            Pedersen, PedersenBase, PedersenCommitment, PedersenDecryptHandle, PedersenOpening,
        },
    },
    crate::errors::ProofError,
    arrayref::{array_ref, array_refs},
    curve25519_dalek::{
        ristretto::{CompressedRistretto, RistrettoPoint},
        scalar::Scalar,
        field::FieldElement,
    },
    serde::{Deserialize, Serialize},
    std::{
        convert::{TryInto, TryFrom},
        fmt,
    },
    subtle::{Choice, ConstantTimeEq},
    zeroize::Zeroize,
};
#[cfg(not(target_arch = "bpf"))]
use {
    rand_core::{OsRng, CryptoRng, RngCore},
    sha3::Sha3_512,
    solana_sdk::{
        pubkey::Pubkey,
        signature::Signature,
        signer::{Signer, SignerError},
    },
    std::{
        fs::{self, File, OpenOptions},
        io::{Read, Write},
        path::Path,
    },
};

struct ElGamal;
impl ElGamal {
    /// The function generates the public and secret keys for ElGamal encryption from the provided
    /// randomness generator
    #[cfg(not(target_arch = "bpf"))]
    #[allow(non_snake_case)]
    fn keygen<T: RngCore + CryptoRng>(rng: &mut T) -> ElGamalKeypair {
        // sample a non-zero scalar
        let mut s: Scalar;
        loop {
            s = Scalar::random(rng);

            if s != Scalar::zero() {
                break;
            }
        }

        Self::keygen_with_scalar(s)
    }

    /// Generates the public and secret keys for ElGamal encryption from a non-zero Scalar
    #[cfg(not(target_arch = "bpf"))]
    #[allow(non_snake_case)]
    fn keygen_with_scalar(s: Scalar) -> ElGamalKeypair {
        assert!(s != Scalar::zero());

        let H = PedersenBase::default().H;
        let P = s.invert() * H;

        ElGamalKeypair {
            public: ElGamalPubkey(P),
            secret: ElGamalSecretKey(s),
        }
    }

    /// On input a public key and a message to be encrypted, the function
    /// returns an ElGamal ciphertext of the message under the public key.
    #[cfg(not(target_arch = "bpf"))]
    fn encrypt<T: Into<CipherKey>>(public: &ElGamalPubkey, amount: T) -> ElGamalCiphertext {
        let (message_comm, open) = Pedersen::new(amount);
        let decrypt_handle = public.decrypt_handle(&open);

        ElGamalCiphertext {
            message_comm,
            decrypt_handle,
        }
    }

    /// On input a public key, message, and Pedersen opening, the function
    /// returns an ElGamal ciphertext of the message under the public key using
    /// the opening.
    fn encrypt_with<T: Into<CipherKey>>(
        public: &ElGamalPubkey,
        amount: T,
        open: &PedersenOpening,
    ) -> ElGamalCiphertext {
        let message_comm = Pedersen::with(amount, open);
        let decrypt_handle = public.decrypt_handle(open);

        ElGamalCiphertext {
            message_comm,
            decrypt_handle,
        }
    }

    /// On input a secret key and a ciphertext, the function decrypts the ciphertext.
    fn decrypt(secret: &ElGamalSecretKey, ct: &ElGamalCiphertext) -> Result<CipherKey, ProofError> {
        let ElGamalSecretKey(s) = secret;
        let ElGamalCiphertext {
            message_comm,
            decrypt_handle,
        } = ct;

        let Q = (message_comm.get_point() - s * decrypt_handle.get_point()).0;
        for P in &[Q, Q + curve25519_dalek::constants::EIGHT_TORSION[1]] {
            let J = ElGamal::ristretto_to_jacobi_isogeny(
                &RistrettoPoint(*P)
            );

            for Jp in &J.coset() {
                let inv = ElGamal::jacobi_elligator_inv(&Jp);
                let r = match inv {
                    Some(r) => r,
                    _ => continue,
                };

                // either the positive or negative root
                for rp in &[r, -&r] {
                    let bytes = rp.to_bytes();
                    match CipherKey::try_from(bytes) {
                        Ok(ck) => {
                            return Ok(ck)
                        },
                        _ => {},
                    }
                }
            }
        }

        Err(ProofError::InconsistentCTData)
    }

    /// (x, y) -> (s, t)
    pub fn ristretto_to_jacobi_isogeny(
        r: &RistrettoPoint,
    ) -> JacobiPoint {
        let z_inv = r.0.Z.invert();
        let x = &r.0.X * &z_inv;
        let y = &r.0.Y * &z_inv;

        let x_sq = x.square();

        let Ns = &curve25519_dalek::constants::SQRT_AD_MINUS_ONE * &(&x * &y);
        let Nt = &y.square() - &x_sq;

        let d_inv = (&FieldElement::one() + &x_sq).invert();

        let st_dub = JacobiPoint {
            X: &Ns * &d_inv,
            Y: &Nt * &d_inv,
        };

        // At this point we've applied the dual `dot` isogeny on (s, t) which is equivalent to
        // 2-multiplication on the corresponding Jacobi point. i.e, we can also mimic this if we
        // had the original (s, t) by doing Jacobi doubling in affine coordinates. Just reverse the
        // doubling

        let two = &Scalar::one() + &Scalar::one();
        st_dub.mul(&two.invert())
    }

    pub fn jacobi_elligator_inv(
        p: &JacobiPoint,
    ) -> Option<FieldElement> {
        use curve25519_dalek::constants;

        let one = FieldElement::one();

        let a2 = &constants::MINUS_ONE;
        let d2 = &constants::EDWARDS_D;

        // (a + d) * s^2 + c * (t + 1) * (d - a)
        let s = &p.X;
        let t = &p.Y;

        let t1 = &(a2 + d2) * &s.square();

        use subtle::ConditionallyNegatable;
        let mut t2 = &(t + &one) * &(d2 - a2);
        t2.conditional_negate(s.is_negative());

        let r = &(&t1 + &t2) * &(&t1 - &t2).invert();

        let i = &constants::SQRT_M1;
        let (r_i_is_sq, r_0) = FieldElement::sqrt_ratio_i(&r, i);

        if r_i_is_sq.unwrap_u8() == 1u8 {
            Some(r_0)
        } else {
            None
        }
    }
}

// Jacobi quartic J_{a_1^2, a_1 - 2 d_1}
// a.k.a          J_{a_2^2, -a_2 \quot{a_2 + d_2}{a_2 - d_2}}
#[derive(Copy, Clone, Debug)]
pub struct JacobiPoint {
    pub X: FieldElement,
    pub Y: FieldElement,
}

impl JacobiPoint {
    pub fn add(&self, other: &JacobiPoint) -> JacobiPoint {
        // constant `A` for this Jacobi quartic
        let a2 = &curve25519_dalek::constants::MINUS_ONE;
        let d2 = &curve25519_dalek::constants::EDWARDS_D;
        let A = &(-&(a2 * &(a2 + d2))) * &(a2 - d2).invert();

        let x1 = &self.X;
        let y1 = &self.Y;
        let x2 = &other.X;
        let y2 = &other.Y;

        let xx = x1 * x2;
        let yy = y1 * y2;

        let one_minus_xx_sq = &FieldElement::one() - &xx.square();
        let one_plus_xx_sq = &FieldElement::one() + &xx.square();
        let two = &FieldElement::one() + &FieldElement::one();

        let xr = &(&(x1 * y2) + &(y1 * x2)) * &one_minus_xx_sq.invert();
        let yr = &(
            &(
                &one_plus_xx_sq
                * &(&yy + &(&(&A + &A) * &xx))
            )
            + &(
                &(&two * &xx)
                * &(&x1.square() + &x2.square())
            )
        ) * &one_minus_xx_sq.square().invert();

        JacobiPoint { X: xr, Y: yr }
    }

    // not constant time
    pub fn mul(&self, scalar: &Scalar) -> JacobiPoint {
        // identity?...
        let mut res = JacobiPoint {
            X: FieldElement::zero(),
            Y: FieldElement::one(),
        };

        let bits = scalar.bits();

        for bit in bits.iter().rev() {
            res = res.add(&res);
            if *bit == 1 {
                res = res.add(self);
            }
        }
        res
    }

    pub fn coset(&self) -> [JacobiPoint; 4] {
        let JacobiPoint { X: s, Y: t } = *self;
        let a_s_inv = &FieldElement::one() * &s.invert();
        let a_t_s_sq_inv = &t * &s.square().invert();
        [
            JacobiPoint{ X: s, Y: t },
            JacobiPoint{ X: -&s, Y: -&t },
            JacobiPoint{ X: a_s_inv, Y: -&a_t_s_sq_inv },
            JacobiPoint{ X: -&a_s_inv, Y: a_t_s_sq_inv },
        ]
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CipherKey(pub [u8; 24]);

impl CipherKey {
    #[cfg(not(target_arch = "bpf"))]
    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut bytes = [0u8; 24];
        rng.fill_bytes(&mut bytes);
        CipherKey(bytes)
    }
}

impl Eq for CipherKey {}
impl PartialEq for CipherKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl TryFrom<[u8; 32]> for CipherKey {
    type Error = ProofError;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        // TODO: why does the high bit flip sometimes? might just be modulo size...
        if bytes[24..31] != [0; 7] || bytes[31] & 0x7F != 0 {
            Err(ProofError::InconsistentCTData)
        } else {
            Ok(CipherKey(
                bytes[..24].try_into()
                    .map_err(|_| ProofError::InconsistentCTData)?
                )
            )
        }
    }
}

impl From<u32> for CipherKey {
    fn from(x: u32) -> CipherKey {
        use byteorder::{ByteOrder, LittleEndian};
        let mut bytes = [0u8; 24];
        LittleEndian::write_u32(&mut bytes, x);
        CipherKey(bytes)
    }
}

impl From<u64> for CipherKey {
    fn from(x: u64) -> CipherKey {
        use byteorder::{ByteOrder, LittleEndian};
        let mut bytes = [0u8; 24];
        LittleEndian::write_u64(&mut bytes, x);
        CipherKey(bytes)
    }
}

/// A (twisted) ElGamal encryption keypair.
#[derive(PartialEq, Debug)]
pub struct ElGamalKeypair {
    /// The public half of this keypair.
    pub public: ElGamalPubkey,
    /// The secret half of this keypair.
    pub secret: ElGamalSecretKey,
}

impl ElGamalKeypair {
    /// Generates the public and secret keys for ElGamal encryption from Ed25519 signing key and an
    /// address.
    #[cfg(not(target_arch = "bpf"))]
    #[allow(non_snake_case)]
    pub fn new(signer: &dyn Signer, address: &Pubkey) -> Result<Self, SignerError> {
        ElGamalSecretKey::new(signer, address)
            .map(|sk| ElGamal::keygen_with_scalar(sk.get_scalar()))
    }

    #[cfg(not(target_arch = "bpf"))]
    pub fn keygen_with_scalar(scalar: Scalar) -> Self {
        ElGamal::keygen_with_scalar(scalar)
    }

    /// Generates the public and secret keys for ElGamal encryption.
    #[cfg(not(target_arch = "bpf"))]
    #[allow(clippy::new_ret_no_self)]
    pub fn default() -> Self {
        Self::with(&mut OsRng) // using OsRng for now
    }

    /// On input a randomness generator, the function generates the public and
    /// secret keys for ElGamal encryption.
    #[cfg(not(target_arch = "bpf"))]
    #[allow(non_snake_case)]
    pub fn with<T: RngCore + CryptoRng>(rng: &mut T) -> Self {
        ElGamal::keygen(rng)
    }

    pub fn to_bytes(&self) -> [u8; 64] {
        let mut bytes = self.public.to_bytes().to_vec();
        bytes.extend(&self.secret.to_bytes());
        bytes.try_into().expect("incorrect length")
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            public: ElGamalPubkey::from_bytes(bytes[..32].try_into().ok()?)?,
            secret: ElGamalSecretKey::from_bytes(bytes[32..].try_into().ok()?)?,
        })
    }

    /// Reads a JSON-encoded keypair from a `Reader` implementor
    #[cfg(not(target_arch = "bpf"))]
    pub fn read_json<R: Read>(reader: &mut R) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes: Vec<u8> = serde_json::from_reader(reader)?;
        Self::from_bytes(&bytes).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "Invalid ElGamalKeypair").into()
        })
    }

    /// Reads keypair from a file
    #[cfg(not(target_arch = "bpf"))]
    pub fn read_json_file<F: AsRef<Path>>(path: F) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path.as_ref())?;
        Self::read_json(&mut file)
    }

    /// Writes to a `Write` implementer with JSON-encoding
    #[cfg(not(target_arch = "bpf"))]
    pub fn write_json<W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = self.to_bytes();
        let json = serde_json::to_string(&bytes.to_vec())?;
        writer.write_all(&json.clone().into_bytes())?;
        Ok(json)
    }

    /// Write keypair to a file with JSON-encoding
    #[cfg(not(target_arch = "bpf"))]
    pub fn write_json_file<F: AsRef<Path>>(
        &self,
        outfile: F,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let outfile = outfile.as_ref();

        if let Some(outdir) = outfile.parent() {
            fs::create_dir_all(outdir)?;
        }

        let mut f = {
            #[cfg(not(unix))]
            {
                OpenOptions::new()
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                OpenOptions::new().mode(0o600)
            }
        }
        .write(true)
        .truncate(true)
        .create(true)
        .open(outfile)?;

        self.write_json(&mut f)
    }
}

/// Public key for the ElGamal encryption scheme.
#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct ElGamalPubkey(RistrettoPoint);
impl ElGamalPubkey {
    /// Derive the `ElGamalPubkey` that uniquely corresponds to an `ElGamalSecretKey`
    #[allow(non_snake_case)]
    pub fn new(secret: &ElGamalSecretKey) -> Self {
        let H = PedersenBase::default().H;
        ElGamalPubkey(secret.0 * H)
    }

    pub fn get_point(&self) -> RistrettoPoint {
        self.0
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.compress().to_bytes()
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Option<ElGamalPubkey> {
        Some(ElGamalPubkey(
            CompressedRistretto::from_slice(bytes).decompress()?,
        ))
    }

    /// Utility method for code ergonomics.
    #[cfg(not(target_arch = "bpf"))]
    pub fn encrypt<T: Into<CipherKey>>(&self, msg: T) -> ElGamalCiphertext {
        ElGamal::encrypt(self, msg)
    }

    /// Utility method for code ergonomics.
    pub fn encrypt_with<T: Into<CipherKey>>(
        &self,
        msg: T,
        open: &PedersenOpening,
    ) -> ElGamalCiphertext {
        ElGamal::encrypt_with(self, msg, open)
    }

    /// Generate a decryption token from an ElGamal public key and a Pedersen
    /// opening.
    pub fn decrypt_handle(self, open: &PedersenOpening) -> PedersenDecryptHandle {
        PedersenDecryptHandle::new(&self, open)
    }
}

impl From<RistrettoPoint> for ElGamalPubkey {
    fn from(point: RistrettoPoint) -> ElGamalPubkey {
        ElGamalPubkey(point)
    }
}

impl fmt::Display for ElGamalPubkey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", base64::encode(self.to_bytes()))
    }
}

/// Secret key for the ElGamal encryption scheme.
#[derive(Serialize, Deserialize, Debug, Zeroize)]
#[zeroize(drop)]
pub struct ElGamalSecretKey(Scalar);
impl ElGamalSecretKey {
    #[cfg(not(target_arch = "bpf"))]
    pub fn new(signer: &dyn Signer, address: &Pubkey) -> Result<Self, SignerError> {
        let message = format!(
            "ElGamalSecretKey:{}:{}",
            bs58::encode(signer.try_pubkey()?).into_string(),
            bs58::encode(address).into_string(),
        );
        let signature = signer.try_sign_message(message.as_bytes())?;

        // Some `Signer` implementations return the default signature, which is not suitable for
        // use as key material
        if signature == Signature::default() {
            Err(SignerError::Custom("Rejecting default signature".into()))
        } else {
            Ok(ElGamalSecretKey(Scalar::hash_from_bytes::<Sha3_512>(
                signature.as_ref(),
            )))
        }
    }

    pub fn get_scalar(&self) -> Scalar {
        self.0
    }

    /// Utility method for code ergonomics.
    pub fn decrypt(&self, ct: &ElGamalCiphertext) -> Result<CipherKey, ProofError> {
        ElGamal::decrypt(self, ct)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Option<ElGamalSecretKey> {
        Scalar::from_canonical_bytes(bytes).map(ElGamalSecretKey)
    }
}

impl From<Scalar> for ElGamalSecretKey {
    fn from(scalar: Scalar) -> ElGamalSecretKey {
        ElGamalSecretKey(scalar)
    }
}

impl Eq for ElGamalSecretKey {}
impl PartialEq for ElGamalSecretKey {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1u8
    }
}
impl ConstantTimeEq for ElGamalSecretKey {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

/// Ciphertext for the ElGamal encryption scheme.
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct ElGamalCiphertext {
    pub message_comm: PedersenCommitment,
    pub decrypt_handle: PedersenDecryptHandle,
}
impl ElGamalCiphertext {
    #[allow(clippy::wrong_self_convention)]
    pub fn to_bytes(&self) -> [u8; 64] {
        let mut bytes = [0u8; 64];

        bytes[..32].copy_from_slice(self.message_comm.get_point().compress().as_bytes());
        bytes[32..].copy_from_slice(self.decrypt_handle.get_point().compress().as_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<ElGamalCiphertext> {
        let bytes = array_ref![bytes, 0, 64];
        let (message_comm, decrypt_handle) = array_refs![bytes, 32, 32];

        let message_comm = CompressedRistretto::from_slice(message_comm).decompress()?;
        let decrypt_handle = CompressedRistretto::from_slice(decrypt_handle).decompress()?;

        Some(ElGamalCiphertext {
            message_comm: PedersenCommitment(message_comm),
            decrypt_handle: PedersenDecryptHandle(decrypt_handle),
        })
    }

    /// Utility method for code ergonomics.
    pub fn decrypt(&self, secret: &ElGamalSecretKey) -> Result<CipherKey, ProofError> {
        ElGamal::decrypt(secret, self)
    }
}

impl From<(PedersenCommitment, PedersenDecryptHandle)> for ElGamalCiphertext {
    fn from((comm, handle): (PedersenCommitment, PedersenDecryptHandle)) -> Self {
        ElGamalCiphertext {
            message_comm: comm,
            decrypt_handle: handle,
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::encryption::pedersen::Pedersen,
        solana_sdk::{signature::Keypair, signer::null_signer::NullSigner},
    };

    #[test]
    fn test_elligator_inv() {
        let bytes: [[u8;32]; 16] = [
            [184, 249, 135, 49, 253, 123, 89, 113, 67, 160, 6, 239, 7, 105, 211, 41, 192, 249, 185, 57, 9, 102, 70, 198, 15, 127, 7, 26, 160, 102, 134, 71],
            [229, 14, 241, 227, 75, 9, 118, 60, 128, 153, 226, 21, 183, 217, 91, 136, 98, 0, 231, 156, 124, 77, 82, 139, 142, 134, 164, 169, 169, 62, 250, 52],
            [115, 109, 36, 220, 180, 223, 99, 6, 204, 169, 19, 29, 169, 68, 84, 23, 21, 109, 189, 149, 127, 205, 91, 102, 172, 35, 112, 35, 134, 69, 186, 34],
            [16, 49, 96, 107, 171, 199, 164, 9, 129, 16, 64, 62, 241, 63, 132, 173, 209, 160, 112, 215, 105, 50, 157, 81, 253, 105, 1, 154, 229, 25, 120, 83],
            [156, 131, 161, 162, 236, 251, 5, 187, 167, 171, 17, 178, 148, 210, 90, 207, 86, 21, 79, 161, 167, 215, 234, 1, 136, 242, 182, 248, 38, 85, 79, 86],
            [251, 177, 124, 54, 18, 101, 75, 235, 245, 186, 19, 46, 133, 157, 229, 64, 10, 136, 181, 185, 78, 144, 254, 167, 137, 49, 107, 10, 61, 10, 21, 25],
            [232, 193, 20, 68, 240, 77, 186, 77, 183, 40, 44, 86, 150, 31, 198, 212, 76, 81, 3, 217, 197, 8, 126, 128, 126, 152, 164, 208, 153, 44, 189, 77],
            [173, 229, 149, 177, 37, 230, 30, 69, 61, 56, 172, 190, 219, 115, 167, 194, 71, 134, 59, 75, 28, 244, 118, 26, 162, 97, 64, 16, 15, 189, 30, 64],
            [106, 71, 61, 107, 250, 117, 42, 151, 91, 202, 212, 100, 52, 188, 190, 21, 125, 218, 31, 18, 253, 241, 160, 133, 57, 242, 3, 164, 189, 68, 111, 75],
            [112, 204, 182, 90, 220, 198, 120, 73, 173, 107, 193, 17, 227, 40, 162, 36, 150, 141, 235, 55, 172, 183, 12, 39, 194, 136, 43, 153, 244, 118, 91, 89],
            [111, 24, 203, 123, 254, 189, 11, 162, 51, 196, 163, 136, 204, 143, 10, 222, 33, 112, 81, 205, 34, 35, 8, 66, 90, 6, 164, 58, 170, 177, 34, 25],
            [225, 183, 30, 52, 236, 82, 6, 183, 109, 25, 227, 181, 25, 82, 41, 193, 80, 77, 161, 80, 242, 203, 79, 204, 136, 245, 131, 110, 237, 106, 3, 58],
            [207, 246, 38, 56, 30, 86, 176, 90, 27, 200, 61, 42, 221, 27, 56, 210, 79, 178, 189, 120, 68, 193, 120, 167, 77, 185, 53, 197, 124, 128, 191, 126],
            [1, 136, 215, 80, 240, 46, 63, 147, 16, 244, 230, 207, 82, 189, 74, 50, 106, 169, 138, 86, 30, 131, 214, 202, 166, 125, 251, 228, 98, 24, 36, 21],
            [210, 207, 228, 56, 155, 116, 207, 54, 84, 195, 251, 215, 249, 199, 116, 75, 109, 239, 196, 251, 194, 246, 252, 228, 70, 146, 156, 35, 25, 39, 241, 4],
            [34, 116, 123, 9, 8, 40, 93, 189, 9, 103, 57, 103, 66, 227, 3, 2, 157, 107, 134, 219, 202, 74, 230, 154, 78, 107, 219, 195, 214, 14, 84, 80],
        ];

        for i in 0..16 {
            let r_0 = FieldElement::from_bytes(&bytes[i]);
            println!("r_0 {:?}", r_0.to_bytes());

            let Q = RistrettoPoint::elligator_ristretto_flavor(&r_0);

            let mut found = 0;
            {
                let p = ElGamal::ristretto_to_jacobi_isogeny(&RistrettoPoint(Q.0));
                for pc in p.coset() {
                    if ElGamal::jacobi_elligator_inv(&pc).map(|r| (-&r).to_bytes()) == Some(bytes[i]) {
                        println!("DECODED NEG {}", i);
                        found += 1;
                    }
                    if ElGamal::jacobi_elligator_inv(&pc).map(|r| r.to_bytes()) == Some(bytes[i]) {
                        println!("DECODED POS {}", i);
                        found += 1;
                    }
                }
            }

            let Qp = Q.0 + curve25519_dalek::constants::EIGHT_TORSION[1];

            {
                let p = ElGamal::ristretto_to_jacobi_isogeny(&RistrettoPoint(Qp));
                for pc in p.coset() {
                    if ElGamal::jacobi_elligator_inv(&pc).map(|r| (-&r).to_bytes()) == Some(bytes[i]) {
                        println!("DECODED TOR NEG {}", i);
                        found += 1;
                    }
                    if ElGamal::jacobi_elligator_inv(&pc).map(|r| r.to_bytes()) == Some(bytes[i]) {
                        println!("DECODED TOR POS {}", i);
                        found += 1;
                    }
                }
            }

            assert_eq!(found, 1, "Did not find exactly 1 decoding!");
        }
    }

    #[test]
    fn test_encrypt_decrypt_correctness() {
        let ElGamalKeypair { public, secret } = ElGamalKeypair::default();
        let msg: u32 = 57;
        let ct = ElGamal::encrypt(&public, msg);

        let expected = CipherKey::from(msg);
        assert_eq!(Ok(expected), ElGamal::decrypt(&secret, &ct));
    }

    #[test]
    fn test_decrypt_handle() {
        let ElGamalKeypair {
            public: pk_1,
            secret: sk_1,
        } = ElGamalKeypair::default();
        let ElGamalKeypair {
            public: pk_2,
            secret: sk_2,
        } = ElGamalKeypair::default();

        let msg: u32 = 77;
        let (comm, open) = Pedersen::new(msg);

        let decrypt_handle_1 = pk_1.decrypt_handle(&open);
        let decrypt_handle_2 = pk_2.decrypt_handle(&open);

        let ct_1: ElGamalCiphertext = (comm, decrypt_handle_1).into();
        let ct_2: ElGamalCiphertext = (comm, decrypt_handle_2).into();

        let expected = CipherKey::from(msg);
        assert_eq!(Ok(expected), sk_1.decrypt(&ct_1));
        assert_eq!(Ok(expected), sk_2.decrypt(&ct_2));
    }

    #[test]
    fn test_serde_ciphertext() {
        let ElGamalKeypair { public, secret: _ } = ElGamalKeypair::default();
        let msg: u64 = 77;
        let ct = public.encrypt(msg);

        let encoded = bincode::serialize(&ct).unwrap();
        let decoded: ElGamalCiphertext = bincode::deserialize(&encoded).unwrap();

        assert_eq!(ct, decoded);
    }

    #[test]
    fn test_serde_pubkey() {
        let ElGamalKeypair { public, secret: _ } = ElGamalKeypair::default();

        let encoded = bincode::serialize(&public).unwrap();
        let decoded: ElGamalPubkey = bincode::deserialize(&encoded).unwrap();

        assert_eq!(public, decoded);
    }

    #[test]
    fn test_serde_secretkey() {
        let ElGamalKeypair { public: _, secret } = ElGamalKeypair::default();

        let encoded = bincode::serialize(&secret).unwrap();
        let decoded: ElGamalSecretKey = bincode::deserialize(&encoded).unwrap();

        assert_eq!(secret, decoded);
    }

    fn tmp_file_path(name: &str) -> String {
        use std::env;
        let out_dir = env::var("FARF_DIR").unwrap_or_else(|_| "farf".to_string());
        let keypair = ElGamalKeypair::default();
        format!("{}/tmp/{}-{}", out_dir, name, keypair.public)
    }

    #[test]
    fn test_write_keypair_file() {
        let outfile = tmp_file_path("test_write_keypair_file.json");
        let serialized_keypair = ElGamalKeypair::default().write_json_file(&outfile).unwrap();
        let keypair_vec: Vec<u8> = serde_json::from_str(&serialized_keypair).unwrap();
        assert!(Path::new(&outfile).exists());
        assert_eq!(
            keypair_vec,
            ElGamalKeypair::read_json_file(&outfile)
                .unwrap()
                .to_bytes()
                .to_vec()
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                File::open(&outfile)
                    .expect("open")
                    .metadata()
                    .expect("metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        fs::remove_file(&outfile).unwrap();
    }

    #[test]
    fn test_write_keypair_file_overwrite_ok() {
        let outfile = tmp_file_path("test_write_keypair_file_overwrite_ok.json");

        ElGamalKeypair::default().write_json_file(&outfile).unwrap();
        ElGamalKeypair::default().write_json_file(&outfile).unwrap();
    }

    #[test]
    fn test_write_keypair_file_truncate() {
        let outfile = tmp_file_path("test_write_keypair_file_truncate.json");

        ElGamalKeypair::default().write_json_file(&outfile).unwrap();
        ElGamalKeypair::read_json_file(&outfile).unwrap();

        // Ensure outfile is truncated
        {
            let mut f = File::create(&outfile).unwrap();
            f.write_all(String::from_utf8([b'a'; 2048].to_vec()).unwrap().as_bytes())
                .unwrap();
        }
        ElGamalKeypair::default().write_json_file(&outfile).unwrap();
        ElGamalKeypair::read_json_file(&outfile).unwrap();
    }

    #[test]
    fn test_secret_key_new() {
        let keypair1 = Keypair::new();
        let keypair2 = Keypair::new();

        assert_ne!(
            ElGamalSecretKey::new(&keypair1, &Pubkey::default())
                .unwrap()
                .0,
            ElGamalSecretKey::new(&keypair2, &Pubkey::default())
                .unwrap()
                .0,
        );

        let null_signer = NullSigner::new(&Pubkey::default());
        assert!(ElGamalSecretKey::new(&null_signer, &Pubkey::default()).is_err());
    }
}
