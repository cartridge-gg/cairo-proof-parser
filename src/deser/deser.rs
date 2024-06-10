use std::ops::{AddAssign, MulAssign};

use serde::de::{self, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use starknet_crypto::FieldElement;

use super::errors::{Error, Result};

pub struct Deserializer<'de> {
    input: &'de [FieldElement],
}

impl<'de> Deserializer<'de> {
    pub fn peek(&self) -> Result<FieldElement> {
        self.input
            .first()
            .map(|x| x.clone())
            .ok_or(Error::NoDataLeft)
    }

    pub fn take(&mut self) -> Result<FieldElement> {
        let el = self.peek()?;
        self.input = &self.input[1..];

        Ok(el)
    }

    pub fn from_felts(input: &'de Vec<FieldElement>) -> Self {
        Deserializer { input }
    }
}

impl<'de> Deserializer<'de> {
    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + From<u8>,
    {
        todo!("parse_unsigned")
    }
}

pub fn from_felts<'a, T>(s: &'a Vec<FieldElement>) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_felts(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        // Err(Error::DataLeft) // TODO: This should be hard fall.
        Ok(t)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let value = self
            .take()?
            .to_string()
            .parse::<u64>()
            .map_err(|_| Error::ValueExceededRange)?;

        visitor.visit_u64(value)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.take()?.to_string())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as JSON arrays of bytes. Handle that representation here.
    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(DeserSeq::new(self))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(DeserSeq::new_with_len(self, len))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // Structs look just like maps in JSON.
    //
    // Notice the `fields` parameter - a "struct" in the Serde data model means
    // that the `Deserialize` implementation is required to know what the fields
    // are before even looking at the input data. Any key-value pairing in which
    // the fields cannot be known ahead of time is probably a map.
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(DeserStruct::new(self, fields))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}

struct DeserStruct<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    fields: &'static [&'static str],
    index: usize,
}

impl<'a, 'de> DeserStruct<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, fields: &'static [&'static str]) -> Self {
        Self {
            de,
            fields,
            index: 0,
        }
    }
}

impl<'a, 'de> MapAccess<'de> for DeserStruct<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.index < self.fields.len() {
            seed.deserialize(self.fields[self.index].into_deserializer())
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        // Deserialize the value for the current field
        let value = seed.deserialize(&mut *self.de)?;
        self.index += 1;
        Ok(value)
    }
}

struct DeserSeq<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    left: Option<usize>,
}

impl<'a, 'de> DeserSeq<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        DeserSeq { de, left: None }
    }

    fn new_with_len(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        println!("len: {}", len);
        DeserSeq {
            de,
            left: Some(len),
        }
    }
}

impl<'de, 'a> SeqAccess<'de> for DeserSeq<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(left) = self.left {
            Ok(if left > 0 {
                self.left = Some(left - 1);
                Some(seed.deserialize(&mut *self.de)?)
            } else {
                None
            })
        } else {
            let len = self
                .de
                .take()?
                .to_string()
                .parse::<usize>()
                .map_err(|_| Error::InvalidArrayLen)?;

            self.left = Some(len);
            self.next_element_seed(seed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct Basic {
        a: FieldElement,
        b: FieldElement,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct Nested {
        a: FieldElement,
        b: Basic,
        c: FieldElement,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct WithSequence {
        a: Vec<FieldElement>,
        b: FieldElement,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct WithArray {
        a: [FieldElement; 2],
        b: FieldElement,
    }

    #[test]
    fn test_deser_basic() -> Result<()> {
        let de: Basic = from_felts(&vec![1u64.into(), 2u64.into()])?;
        let expected = Basic {
            a: 1u64.into(),
            b: 2u64.into(),
        };

        assert_eq!(de, expected);

        Ok(())
    }

    #[test]
    fn test_deser_nested() -> Result<()> {
        let de: Nested = from_felts(&vec![1u64.into(), 11u64.into(), 12u64.into(), 2u64.into()])?;
        let expected = Nested {
            a: 1u64.into(),
            b: Basic {
                a: 11u64.into(),
                b: 12u64.into(),
            },
            c: 2u64.into(),
        };

        assert_eq!(de, expected);

        Ok(())
    }

    #[test]
    fn test_deser_seq() -> Result<()> {
        let de: WithSequence =
            from_felts(&vec![2u64.into(), 11u64.into(), 12u64.into(), 2u64.into()])?;
        let expected = WithSequence {
            a: vec![11u64.into(), 12u64.into()],
            b: 2u64.into(),
        };

        assert_eq!(de, expected);

        Ok(())
    }

    #[test]
    fn test_deser_arr() -> Result<()> {
        let de: WithArray = from_felts(&vec![11u64.into(), 12u64.into(), 2u64.into()])?;
        let expected = WithArray {
            a: [11u64.into(), 12u64.into()],
            b: 2u64.into(),
        };

        assert_eq!(de, expected);

        Ok(())
    }
}
