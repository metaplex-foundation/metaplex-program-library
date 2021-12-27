
extern crate bincode;
extern crate curve25519_dalek;
extern crate private_metadata;
#[macro_use]
extern crate serde;
extern crate sha3;
extern crate solana_sdk;
extern crate wasm_bindgen;

use curve25519_dalek::scalar::Scalar;
use private_metadata::encryption::elgamal::ElGamalKeypair;
use serde::de::{Deserialize, Deserializer, Visitor, SeqAccess, MapAccess, Error};
use serde::ser::{Serialize, SerializeStruct, Serializer, SerializeTuple}; // traits
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

#[wasm_bindgen(module = "loglevel")]
extern "C" {
    fn debug(s: &str);
}

#[wasm_bindgen]
pub fn elgamal_keypair_new(signer: &JsValue, address: &JsValue) -> JsValue {
    debug(&format!("Inputs\n\tsigner: {:?}\n\taddress: {:?}", signer, address));

    let signer_bytes: KeypairBytes = signer.into_serde().unwrap();
    let signer = Keypair::from_bytes(&signer_bytes.bytes).unwrap();
    let address: Pubkey = address.into_serde().unwrap();

    debug(&format!("Processed Inputs"));

    let kp = ElGamalKeypair::new(&signer, &address).unwrap();

    debug(&format!("Finished compute"));

    JsValue::from_serde(&JSElGamalKeypair(kp)).unwrap()
}

#[wasm_bindgen]
pub fn elgamal_keypair_from_signature(signature: &JsValue) -> JsValue {
    debug(&format!("Inputs\n\tsignature: {:?}", signature));

    let signature: Signature = signature.into_serde().unwrap();

    debug(&format!("Processed Inputs"));

    let scalar = Scalar::hash_from_bytes::<Sha3_512>(signature.as_ref());
    let kp = ElGamalKeypair::keygen_with_scalar(scalar);

    debug(&format!("Finished compute"));

    JsValue::from_serde(&JSElGamalKeypair(kp)).unwrap()
}

#[wasm_bindgen]
pub fn elgamal_decrypt_u32(elgamal_keypair: &JsValue, ciphertext: &JsValue) -> JsValue {
    debug(&format!("Inputs\n\telgamal_keypair: {:?}\n\tciphertext: {:?}", elgamal_keypair, ciphertext));

    let elgamal_keypair: JSElGamalKeypair = elgamal_keypair.into_serde().unwrap();
    let ciphertext_bytes: ElGamalCiphertextBytes = ciphertext.into_serde().unwrap();

    debug(&format!("Processed Inputs"));

    let res = elgamal_keypair.0.secret.decrypt_u32(
        &private_metadata::zk_token_elgamal::pod::ElGamalCiphertext(ciphertext_bytes.bytes).try_into().unwrap(),
    ).unwrap();

    debug(&format!("Finished compute"));

    JsValue::from_serde(&res.to_le_bytes()).unwrap()
}
