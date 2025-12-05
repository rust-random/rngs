// Copyright 2018 Developers of the Rand project.
// Copyright 2017-2018 The Rust Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! ISAAC helper functions for 256-element arrays.

// Terrible workaround because arrays with more than 32 elements do not
// implement `AsRef`, `Default`, `Serialize`, `Deserialize`, or any other
// traits for that matter.

pub mod array {
    use crate::RAND_SIZE;
    use core::fmt;
    use serde::{
        Deserialize, Deserializer, Serialize, Serializer,
        de::{self, SeqAccess, Visitor},
    };

    pub fn serialize<T, S>(arr: &[T; RAND_SIZE], ser: S) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut seq = ser.serialize_tuple(RAND_SIZE)?;

        for e in arr.iter() {
            seq.serialize_element(&e)?;
        }

        seq.end()
    }

    #[inline]
    pub fn deserialize<'de, T, D>(de: D) -> Result<[T; RAND_SIZE], D::Error>
    where
        T: Deserialize<'de> + Default + Copy,
        D: Deserializer<'de>,
    {
        use core::marker::PhantomData;
        struct ArrayVisitor<T> {
            _pd: PhantomData<T>,
        }
        impl<'de, T> Visitor<'de> for ArrayVisitor<T>
        where
            T: Deserialize<'de> + Default + Copy,
        {
            type Value = [T; RAND_SIZE];

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Isaac state array")
            }

            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<[T; RAND_SIZE], A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut out = [Default::default(); RAND_SIZE];

                for (i, dst) in out.iter_mut().enumerate() {
                    match seq.next_element()? {
                        Some(val) => *dst = val,
                        None => return Err(de::Error::invalid_length(i, &self)),
                    };
                }

                Ok(out)
            }
        }

        de.deserialize_tuple(RAND_SIZE, ArrayVisitor { _pd: PhantomData })
    }
}

pub mod buffer {
    use crate::RAND_SIZE;
    use rand_core::le::{BlockBuffer, Word};
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

    pub fn serialize<T, S>(buf: &BlockBuffer<T, RAND_SIZE>, ser: S) -> Result<S::Ok, S::Error>
    where
        T: Serialize + Word,
        S: Serializer,
    {
        super::array::serialize(&buf.into_array(), ser)
    }

    #[inline]
    pub fn deserialize<'de, T, D>(de: D) -> Result<BlockBuffer<T, RAND_SIZE>, D::Error>
    where
        T: Deserialize<'de> + Word,
        D: Deserializer<'de>,
    {
        let arr = super::array::deserialize(de)?;
        BlockBuffer::try_from_array(arr)
            .ok_or_else(|| D::Error::custom("Failed to convert array to BlockBuffer"))
    }
}
