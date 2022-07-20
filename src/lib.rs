use std::fmt;

use format::Marker;
use serde::{de::value::BorrowedStrDeserializer, forward_to_deserialize_any, Deserialize};
use traits::{VisitDouble, VisitInt};

mod format;
mod traits;

#[derive(Debug, PartialEq)]
enum ErrorKind {
    #[allow(dead_code)]
    Unimplemented,
    Custom(String),
    Format(format::Error),
}

#[derive(Debug, PartialEq)]
pub struct Error {
    kind: ErrorKind,
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Unimplemented => write!(f, "Unimplemented"),
            ErrorKind::Custom(msg) => write!(f, "Custom: {}", msg),
            ErrorKind::Format(fmt) => write!(f, "Format error: {:?}", fmt),
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self {
            kind: ErrorKind::Custom(msg.to_string()),
        }
    }
}

struct ByteDeserializerSeq<'a, 'de> {
    len: usize,
    inner: &'a mut ByteDeserializer<'de>,
}

impl<'a, 'de> serde::de::SeqAccess<'de> for ByteDeserializerSeq<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.len > 0 {
            self.len -= 1;
            seed.deserialize(&mut *self.inner).map(Some)
        } else {
            Ok(None)
        }
    }
}

struct ByteDeserializerMap<'a, 'de> {
    len: usize,
    next_key: &'de str,
    inner: &'a mut ByteDeserializer<'de>,
}

impl<'a, 'de> serde::de::MapAccess<'de> for ByteDeserializerMap<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.next_key.is_empty() {
            if self.len > 0 {
                self.len -= 1;
                let deserializer = serde::de::value::UsizeDeserializer::new(self.len);
                seed.deserialize(deserializer).map(Some)
            } else {
                Ok(None)
            }
        } else {
            let deserializer = BorrowedStrDeserializer::new(self.next_key);
            seed.deserialize(deserializer).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut *self.inner)?;
        if !self.next_key.is_empty() {
            self.next_key = self.inner.inner.read_string()?;
        }
        Ok(value)
    }
}

pub struct ByteDeserializer<'de> {
    inner: format::Deserializer<'de>,
}

impl<'de> ByteDeserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Self {
            inner: format::Deserializer::new(input),
        }
    }

    fn deserialize_array<V: serde::de::Visitor<'de>>(
        &mut self,
        visitor: V,
    ) -> Result<V::Value, Error> {
        let header = self.inner.read_u29()?;
        let value = (header >> 1) as usize;
        if header & 1 == 0 {
            // array by reference
            unimplemented!()
        } else {
            // dense count
            let first_key = self.inner.read_string()?;
            if first_key.is_empty() {
                // only dense keys => array
                visitor.visit_seq(ByteDeserializerSeq {
                    inner: self,
                    len: value,
                })
            } else {
                visitor.visit_map(ByteDeserializerMap {
                    inner: self,
                    len: value,
                    next_key: first_key,
                })
            }
        }
    }

    fn deserialize_into<V, N: VisitInt, F: VisitDouble>(
        &mut self,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let marker = self.inner.read_marker()?;
        match marker {
            Marker::Undefined => visitor.visit_none(),
            Marker::Null => visitor.visit_none(),
            Marker::False => visitor.visit_bool(false),
            Marker::True => visitor.visit_bool(true),
            Marker::Integer => N::visit_int(visitor, self.inner.read_u29()?),
            Marker::Double => F::visit_double(visitor, self.inner.read_double()?),
            Marker::String => visitor.visit_borrowed_str(self.inner.read_string()?),
            Marker::XmlDoc => todo!(),
            Marker::Date => todo!(),
            Marker::Array => self.deserialize_array(visitor),
            Marker::Object => todo!(),
            Marker::Xml => todo!(),
            Marker::ByteArray => todo!(),
            Marker::VectorInt => todo!(),
            Marker::VectorUInt => todo!(),
            Marker::VectorDouble => todo!(),
            Marker::VectorObject => todo!(),
            Marker::Dictionary => todo!(),
        }
    }
}

impl From<format::Error> for Error {
    fn from(e: format::Error) -> Self {
        Self {
            kind: ErrorKind::Format(e),
        }
    }
}

pub fn deserialize<'de, T: Deserialize<'de>>(input: &'de [u8]) -> Result<T, Error> {
    let mut deserializer = ByteDeserializer::from_bytes(input);
    T::deserialize(&mut deserializer)
}

impl<'de> serde::Deserializer<'de> for &mut ByteDeserializer<'de> {
    type Error = Error;

    forward_to_deserialize_any! { bool str string option unit seq tuple map struct identifier }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_into::<V, u32, f64>(visitor)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_into::<V, i8, i8>(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_into::<V, i16, i16>(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_into::<V, i32, i32>(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_into::<V, i64, i64>(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_into::<V, u8, u8>(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_into::<V, u16, u16>(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_into::<V, u32, u32>(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_into::<V, u64, u64>(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_into::<V, f32, f32>(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_into::<V, f64, f64>(visitor)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    /*fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }*/

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.inner.skip()?;
        visitor.visit_none()
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::{format, Error, ErrorKind};

    const EOS_ERROR: Error = Error {
        kind: ErrorKind::Format(format::Error::EndOfStream),
    };

    #[test]
    fn test_bool() {
        assert_eq!(super::deserialize::<bool>(&[]), Err(EOS_ERROR));
        assert_eq!(super::deserialize::<bool>(&[0x02]), Ok(false));
        assert_eq!(super::deserialize::<bool>(&[0x03]), Ok(true));
    }

    #[test]
    fn test_integer() {
        assert_eq!(super::deserialize(&[0x04, 0x05]), Ok(5u8));
        assert_eq!(super::deserialize(&[0x04, 0x05]), Ok(5u16));
        assert_eq!(super::deserialize(&[0x04, 0x05]), Ok(5u32));
        assert_eq!(super::deserialize(&[0x04, 0x05]), Ok(5u64));
    }

    #[test]
    fn test_double() {
        assert_eq!(super::deserialize(&[0x05, 0, 0, 0, 0, 0, 0, 0, 0]), Ok(0.0));
        assert_eq!(
            super::deserialize(&[0x05, 0, 0, 0, 0, 0, 0, 0, 0]),
            Ok(0.0f32)
        );
        assert_eq!(
            super::deserialize(&[0x05, 0, 0, 0, 0, 0, 0, 0xD0, 0x3F]),
            Ok(0.25)
        );
        assert_eq!(
            super::deserialize(&[0x05, 0, 0, 0, 0, 0, 0, 0xD0, 0x3F]),
            Ok(0.25f32)
        );
        assert_eq!(
            super::deserialize(&[0x05, 0x9A, 0x99, 0x99, 0x99, 0x99, 0x99, 0xB9, 0x3F]),
            Ok(0.1)
        );
    }

    #[test]
    fn test_string() {
        assert_eq!(super::deserialize(b"\x06\x0BHello"), Ok("Hello"));
    }

    #[test]
    fn test_option() {
        assert_eq!(super::deserialize::<Option<u32>>(b"\x00"), Ok(None));
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Test {
        a: u32,
        b: u32,
    }

    #[test]
    fn test_array() {
        assert_eq!(
            super::deserialize::<Vec<u32>>(&[0x09, 0x7, 0x01, 0x04, 1, 0x04, 2, 0x04, 3]),
            Ok(vec![1, 2, 3])
        );
        assert_eq!(
            super::deserialize(&[0x09, 0x1, 0x03, b'a', 0x04, 5, 0x03, b'b', 0x04, 7, 0x01]),
            Ok(Test { a: 5, b: 7 })
        );
    }
}
