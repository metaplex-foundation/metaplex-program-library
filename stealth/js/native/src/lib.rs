
use curve25519_dalek::scalar::Scalar;
use stealth::encryption::elgamal::{CipherKey, ElGamalKeypair, ElGamalPubkey};
use stealth::zk_token_elgamal::pod;
use serde::de::{Deserializer, Visitor, SeqAccess, MapAccess, Error};
use serde::ser::{SerializeStruct, Serializer, SerializeTuple}; // traits
use serde::{Deserialize, Serialize};
use sha3::Sha3_512;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::signature::Signature;
use std::convert::TryInto;
use std::fmt;
use std::marker::PhantomData;
use wasm_bindgen::prelude::*;

trait BigArray<'de>: Sized {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer;
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>;
}

macro_rules! big_array {
    ($($len:expr,)+) => {
        $(
            impl<'de, T> BigArray<'de> for [T; $len]
                where T: Default + Copy + Serialize + Deserialize<'de>
            {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    where S: Serializer
                {
                    let mut seq = serializer.serialize_tuple(self.len())?;
                    for elem in &self[..] {
                        seq.serialize_element(elem)?;
                    }
                    seq.end()
                }

                fn deserialize<D>(deserializer: D) -> Result<[T; $len], D::Error>
                    where D: Deserializer<'de>
                {
                    struct ArrayVisitor<T> {
                        element: PhantomData<T>,
                    }

                    impl<'de, T> Visitor<'de> for ArrayVisitor<T>
                        where T: Default + Copy + Deserialize<'de>
                    {
                        type Value = [T; $len];

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str(concat!("an array of length ", $len))
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<[T; $len], A::Error>
                            where A: SeqAccess<'de>
                        {
                            let mut arr = [T::default(); $len];
                            for i in 0..$len {
                                arr[i] = seq.next_element()?
                                    .ok_or_else(|| Error::invalid_length(i, &self))?;
                            }
                            Ok(arr)
                        }
                    }

                    let visitor = ArrayVisitor { element: PhantomData };
                    deserializer.deserialize_tuple($len, visitor)
                }
            }
        )+
    }
}

big_array! {
    64,
}

#[derive(Serialize, Deserialize, Debug)]
struct KeypairBytes {
    #[serde(with = "BigArray")]
    bytes: [u8; 64],
}

#[derive(Serialize, Deserialize, Debug)]
struct ElGamalCiphertextBytes {
    #[serde(with = "BigArray")]
    bytes: [u8; 64],
}

#[derive(Debug)]
pub struct JSElGamalKeypair(ElGamalKeypair);

impl Serialize for JSElGamalKeypair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("JSElGamalKeypair", 2)?;
        s.serialize_field("public", &self.0.public)?;
        s.serialize_field("secret", &self.0.secret)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for JSElGamalKeypair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Public, Secret }

        struct JSElGamalKeypairVisitor;
        impl<'de> Visitor<'de> for JSElGamalKeypairVisitor
        {
            type Value = JSElGamalKeypair;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct JSElGamalKeypair")
            }

            fn visit_map<V>(self, mut map: V) -> Result<JSElGamalKeypair, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut public = None;
                let mut secret = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Public => {
                            if public.is_some() {
                                return Err(Error::duplicate_field("public"));
                            }
                            public = Some(map.next_value()?);
                        }
                        Field::Secret => {
                            if secret.is_some() {
                                return Err(Error::duplicate_field("secret"));
                            }
                            secret = Some(map.next_value()?);
                        }
                    }
                }
                let public = public.ok_or_else(|| Error::missing_field("public"))?;
                let secret = secret.ok_or_else(|| Error::missing_field("secret"))?;
                Ok(JSElGamalKeypair(ElGamalKeypair { public, secret }))
            }
        }

        const FIELDS: &'static [&'static str] = &["public", "secret"];
        deserializer.deserialize_struct("JSElGamalKeypair", FIELDS, JSElGamalKeypairVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct TransferChunkAccounts {
    pub payer: Pubkey,
    pub instruction_buffer: Pubkey,
    pub input_buffer: Pubkey,
    pub compute_buffer: Pubkey,
}

#[wasm_bindgen(module = "loglevel")]
extern "C" {
    fn debug(s: &str);
}

// non-ref taking?
// using ToString::to_string doesn't seem to work...
fn to_string<T: ToString>(t: T) -> String {
    t.to_string()
}

#[wasm_bindgen]
pub fn elgamal_keypair_new(signer: &JsValue, address: &JsValue) -> JsValue {
    let go = || -> Result<ElGamalKeypair, String> {
        debug(&format!("Inputs\n\tsigner: {:?}\n\taddress: {:?}", signer, address));

        let signer_bytes: KeypairBytes = signer.into_serde().map_err(to_string)?;
        let signer = Keypair::from_bytes(&signer_bytes.bytes).map_err(to_string)?;
        let address: Pubkey = address.into_serde().map_err(to_string)?;

        debug(&format!("Processed Inputs"));

        let kp = ElGamalKeypair::new(&signer, &address).map_err(to_string)?;

        debug(&format!("Finished compute"));

        Ok(kp)
    };

    JsValue::from_serde(&go().map(JSElGamalKeypair)).unwrap()
}

#[wasm_bindgen]
pub fn elgamal_keypair_from_signature(signature: &JsValue) -> JsValue {
    let go = || -> Result<ElGamalKeypair, String> {
        debug(&format!("Inputs\n\tsignature: {:?}", signature));

        let signature: Signature = signature.into_serde().map_err(to_string)?;

        debug(&format!("Processed Inputs"));

        let scalar = Scalar::hash_from_bytes::<Sha3_512>(signature.as_ref());
        let kp = ElGamalKeypair::keygen_with_scalar(scalar);

        debug(&format!("Finished compute"));

        Ok(kp)
    };

    JsValue::from_serde(&go().map(JSElGamalKeypair)).unwrap()
}

#[wasm_bindgen]
pub fn elgamal_decrypt(elgamal_keypair: &JsValue, ciphertext: &JsValue) -> JsValue {
    let go = || -> Result<CipherKey, String> {
        debug(&format!("Inputs\n\telgamal_keypair: {:?}\n\tciphertext: {:?}", elgamal_keypair, ciphertext));

        let elgamal_keypair: JSElGamalKeypair = elgamal_keypair.into_serde().map_err(to_string)?;
        let ciphertext_bytes: ElGamalCiphertextBytes = ciphertext.into_serde().map_err(to_string)?;

        debug(&format!("Processed Inputs"));

        let res = elgamal_keypair.0.secret.decrypt(
            &pod::ElGamalCiphertext(ciphertext_bytes.bytes).try_into().map_err(to_string)?,
        ).map_err(to_string)?;

        debug(&format!("Finished compute"));

        Ok(res)
    };

    JsValue::from_serde(&go().map(|v| v.0)).unwrap()
}

#[wasm_bindgen]
pub fn transfer_chunk_txs(
    elgamal_keypair: &JsValue,
    recipient_elgamal_pubkey: &JsValue,
    ciphertext: &JsValue,
    cipherkey: &JsValue,
    accounts: &JsValue,
) -> JsValue {
    let go = || -> Result<(
            Vec<stealth::instruction::InstructionsAndSignerPubkeys>,
            Vec<u8>,
        ),
        String
    > {
        debug(&format!("\
            Inputs\n\
            \telgamal_keypair: {:?}\n\
            \trecipient_elgamal_pubkey: {:?}\n\
            \tciphertext: {:?}\n\
            \tcipherkey: {:?}\n\
            \taccounts: {:?}\
            ",
            elgamal_keypair,
            recipient_elgamal_pubkey,
            ciphertext,
            cipherkey,
            accounts,
        ));

        let elgamal_keypair: JSElGamalKeypair = elgamal_keypair.into_serde().map_err(to_string)?;
        let recipient_elgamal_pubkey: ElGamalPubkey = recipient_elgamal_pubkey.into_serde().map_err(to_string)?;
        let ciphertext_bytes: ElGamalCiphertextBytes = ciphertext.into_serde().map_err(to_string)?;
        let cipherkey: CipherKey = cipherkey.into_serde().map_err(to_string)?;
        debug(&format!("Processing accounts"));
        let accounts: TransferChunkAccounts = accounts.into_serde().map_err(to_string)?;

        debug(&format!("Processed Inputs"));

        let ct =  pod::ElGamalCiphertext(ciphertext_bytes.bytes).try_into().map_err(to_string)?;

        debug(&format!("Build ct"));

        let transfer = stealth::transfer_proof::TransferData::new(
            &elgamal_keypair.0,
            recipient_elgamal_pubkey,
            cipherkey,
            ct,
        );

        debug(&format!("Built transfer proof"));

        let txs = stealth::instruction::transfer_chunk_slow_proof(
            &accounts.payer,
            &accounts.instruction_buffer,
            &accounts.input_buffer,
            &accounts.compute_buffer,
            &transfer,
            |_| u64::MAX,
        ).map_err(to_string)?;

        debug(&format!("Finished compute"));

        Ok((txs, bytemuck::cast_slice(&[transfer]).to_vec()))
    };

    JsValue::from_serde(&go()).unwrap()
}

#[wasm_bindgen]
pub fn transfer_buffer_len() -> usize {
    use stealth::pod::PodAccountInfo;
    stealth::state::CipherKeyTransferBuffer::get_packed_len()
}
