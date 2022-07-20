use std::str::Utf8Error;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Marker {
    Undefined = 0x00,
    Null = 0x01,
    False = 0x02,
    True = 0x03,
    Integer = 0x04,
    Double = 0x05,
    String = 0x06,
    XmlDoc = 0x07,
    Date = 0x08,
    Array = 0x09,
    Object = 0x0A,
    Xml = 0x0B,
    ByteArray = 0x0C,
    VectorInt = 0x0D,
    VectorUInt = 0x0E,
    VectorDouble = 0x0F,
    VectorObject = 0x10,
    Dictionary = 0x11,
}

impl Marker {
    fn new(value: u8) -> Result<Self, Error> {
        if value < 0x12 {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(Error::InvalidMarker(value))
        }
    }
}

#[derive(Debug, PartialEq)]
pub(super) enum Error {
    InvalidMarker(u8),
    StringDecode(Utf8Error),
    EndOfStream,
    MissingStringReference,
}

impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Self {
        Self::StringDecode(e)
    }
}

pub struct Deserializer<'de> {
    input: std::slice::Iter<'de, u8>,

    string_reference_table: Vec<&'de str>,
}

fn try_split_array_ref<const N: usize>(slice: &[u8]) -> Result<(&[u8; N], &[u8]), Error> {
    if slice.len() < N {
        Err(Error::EndOfStream)
    } else {
        let rest = unsafe { slice.get_unchecked(N..) };
        // SAFETY: a points to [T; N]? Yes it's [T] of length N (checked by split_at)
        Ok((unsafe { &*(slice.as_ptr() as *const [u8; N]) }, rest))
    }
}

impl<'de> Deserializer<'de> {
    pub(super) fn read_byte(&mut self) -> Result<u8, Error> {
        self.input.next().copied().ok_or(Error::EndOfStream)
    }

    pub(super) fn read_marker(&mut self) -> Result<Marker, Error> {
        let byte = self.read_byte()?;
        Marker::new(byte)
    }

    /// 0x00000000 - 0x0000007F : 0xxxxxxx
    /// 0x00000080 - 0x00003FFF : 1xxxxxxx 0xxxxxxx
    /// 0x00004000 - 0x001FFFFF : 1xxxxxxx 1xxxxxxx 0xxxxxxx
    /// 0x00200000 - 0x3FFFFFFF : 1xxxxxxx 1xxxxxxx 1xxxxxxx xxxxxxxx
    /// 0x40000000 - 0xFFFFFFFF : throw range exception
    pub(super) fn read_u29(&mut self) -> Result<u32, Error> {
        let first = self.read_byte()?;
        let mut value = u32::from(first & 0x7F);
        if first >= 0x80 {
            let second = self.read_byte()?;
            value <<= 7;
            value |= u32::from(second & 0x7F);
            if second >= 0x80 {
                let third = self.read_byte()?;
                value <<= 7;
                value |= u32::from(third & 0x7F);
                if third >= 0x80 {
                    let fourth = self.read_byte()?;
                    value <<= 8;
                    value |= u32::from(fourth);
                }
            }
        }
        Ok(value)
    }

    pub(super) fn read_double(&mut self) -> Result<f64, Error> {
        let slice = self.input.as_slice();
        let (double_bytes, rest) = try_split_array_ref(slice)?;
        self.input = rest.iter();
        Ok(f64::from_le_bytes(*double_bytes))
    }

    pub(super) fn read_string(&mut self) -> Result<&'de str, Error> {
        let header = self.read_u29()?;
        let value = (header >> 1) as usize;
        if header & 1 == 0 {
            // by reference
            let string = *(self
                .string_reference_table
                .get(value)
                .ok_or(Error::MissingStringReference)?);
            Ok(string)
        } else if self.input.len() >= value {
            // by value
            let slice = self.input.as_slice();
            let string_bytes = unsafe { slice.get_unchecked(..value) };
            let rest = unsafe { slice.get_unchecked(value..) };
            self.input = rest.iter();
            let string = std::str::from_utf8(string_bytes)?;
            if !string.is_empty() {
                self.string_reference_table.push(string);
            }
            Ok(string)
        } else {
            Err(Error::EndOfStream)
        }
    }

    pub(crate) fn new(input: &'de [u8]) -> Self {
        Self {
            input: input.iter(),
            string_reference_table: Vec::new(),
        }
    }

    pub(crate) fn skip(&mut self) -> Result<(), Error> {
        let marker = self.read_marker()?;
        match marker {
            Marker::Integer => {
                self.read_u29()?;
            }
            Marker::Double => {
                self.input = self
                    .input
                    .as_slice()
                    .get(8..)
                    .ok_or(Error::EndOfStream)?
                    .iter();
            }
            Marker::String => todo!(),
            Marker::XmlDoc => todo!(),
            Marker::Date => todo!(),
            Marker::Array => todo!(),
            Marker::Object => todo!(),
            Marker::Xml => todo!(),
            Marker::ByteArray => todo!(),
            Marker::VectorInt => todo!(),
            Marker::VectorUInt => todo!(),
            Marker::VectorDouble => todo!(),
            Marker::VectorObject => todo!(),
            Marker::Dictionary => todo!(),
            _ => {}
        }
        Ok(())
    }
}
