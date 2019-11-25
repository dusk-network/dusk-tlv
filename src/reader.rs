use crate::Error;

use std::io::{self, Read};

use serde::de::{DeserializeSeed, Deserializer, SeqAccess, Visitor};

/// Optionally consumes an implementation of [`Read`], and fetch n payloads in TLV format from it.
///
/// The payloads can be fetched either via [`TlvReader::reader_to_tlv_len`], or via the iterator
pub struct TlvReader<R>
where
    R: io::Read,
{
    reader: R,
}

impl<R> TlvReader<R>
where
    R: io::Read,
{
    /// [`TlvReader`] constructor
    pub fn new(reader: R) -> Self {
        TlvReader { reader }
    }

    /// Consumes self, and return the inner reader
    pub fn into_inner(self) -> R {
        self.reader
    }

    /// Consumes an implementation of [`Read`], and return the amount of bytes that should be read
    /// to fetch the TLV payload.
    ///
    /// The function will effectively read the bytes to fetch the length, so the reader will be
    /// pointing to the begining of the payload after the call.
    pub fn reader_to_tlv_len(reader: R) -> Result<usize, Error> {
        let mut reader = reader;

        // The first byte defines the type
        let mut tlv_type = [0x0u8];
        reader.read_exact(&mut tlv_type[..])?;

        // Since we always use 0xf`x` format, we just need to extract the least significant byte
        //
        // This can be performed by a simple bitwise operation
        let len = (tlv_type[0] & 0x0f) as usize;

        // The TLV length cannot be bigger than a [`u64`]. Since this value is immensely big, there
        // should be no case when we need more bytes than that.
        //
        // Here, the amount of bytes defined by the type mask will be read.
        let mut tlv_len = [0x00u8; 8];
        reader.read_exact(&mut tlv_len[..len])?;
        let tlv_len = u64::from_le_bytes(tlv_len);

        Ok(tlv_len as usize)
    }

    /// From an implementation of [`Read`], fetch the type, length and write the value to the
    /// provided buf.
    pub fn read_slice(reader: R, buf: &mut [u8]) -> Result<usize, Error> {
        let mut reader = reader;

        let tlv_len = TlvReader::reader_to_tlv_len(&mut reader)?;

        // If the provided length is bigger than the buffer, then the provided buffer cannot
        // contain all the bytes. This verification prevents inconsistent data.
        if buf.len() < tlv_len as usize {
            return Err(Error::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "The buffer is not big enough",
            )));
        }

        // Grant we take all the bytes informed by the type from the reader
        let mut reader = reader.take(tlv_len as u64);
        reader.read_exact(&mut buf[..tlv_len as usize])?;

        Ok(tlv_len)
    }

    /// Read a list of serializable items from the provided reader
    pub fn read_list<L: From<Vec<u8>>>(&mut self) -> Result<Vec<L>, Error> {
        let buf = self.next().ok_or(Error::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes to read the list from the TLV format",
        )))??;

        let mut list = vec![];
        for item in TlvReader::new(buf.as_slice()) {
            let item = item?;
            list.push(L::from(item));
        }

        Ok(list)
    }
}

impl<R> From<R> for TlvReader<R>
where
    R: io::Read,
{
    fn from(reader: R) -> Self {
        TlvReader::new(reader)
    }
}

impl<R> Iterator for TlvReader<R>
where
    R: io::Read,
{
    type Item = Result<Vec<u8>, Error>;

    /// Since we are dealing with I/O, the iterator itself is error-prone. Therefore, the item must
    /// be a [`Result`].
    ///
    /// If the function fails to read the TLV type/length, then it will return [`None`].
    ///
    /// If the type/length is successfully read but we have an I/O error, the function will return a
    /// [`Some(Error)`]
    ///
    /// Otherwise, the payload of the TLV will be returned
    fn next(&mut self) -> Option<Self::Item> {
        let tlv_len = match TlvReader::reader_to_tlv_len(&mut self.reader) {
            Ok(l) => l,
            Err(_) => return None,
        };

        let mut v = Vec::with_capacity(tlv_len);

        let reader = &mut self.reader;
        let mut reader = reader.take(tlv_len as u64);
        let bytes = match reader.read_to_end(&mut v).map_err(|e| e.into()) {
            Ok(b) => b,
            Err(e) => return Some(Err(e)),
        };

        if bytes < tlv_len {
            return Some(Err(Error::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "The reader didnt provide enough bytes for the TLV decoding",
            ))));
        }

        Some(Ok(v))
    }
}

impl<'de, R> SeqAccess<'de> for TlvReader<R>
where
    R: io::Read,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, _seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        unimplemented!()
    }
}

impl<'de, R> Deserializer<'de> for TlvReader<R>
where
    R: io::Read,
{
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0x00u8];
        TlvReader::read_slice(&mut self.reader, &mut buf)?;
        visitor.visit_bool(if buf[0] == 0 { false } else { true })
    }

    fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0x00u8];
        TlvReader::read_slice(&mut self.reader, &mut buf)?;
        visitor.visit_i8(i8::from_le_bytes(buf))
    }

    fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0x00u8; 2];
        TlvReader::read_slice(&mut self.reader, &mut buf)?;
        visitor.visit_i16(i16::from_le_bytes(buf))
    }

    fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0x00u8; 4];
        TlvReader::read_slice(&mut self.reader, &mut buf)?;
        visitor.visit_i32(i32::from_le_bytes(buf))
    }

    fn deserialize_i64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0x00u8; 8];
        TlvReader::read_slice(&mut self.reader, &mut buf)?;
        visitor.visit_i64(i64::from_le_bytes(buf))
    }

    fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0x00u8; 1];
        TlvReader::read_slice(&mut self.reader, &mut buf)?;
        visitor.visit_u8(u8::from_le_bytes(buf))
    }

    fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0x00u8; 2];
        TlvReader::read_slice(&mut self.reader, &mut buf)?;
        visitor.visit_u16(u16::from_le_bytes(buf))
    }

    fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0x00u8; 4];
        TlvReader::read_slice(&mut self.reader, &mut buf)?;
        visitor.visit_u32(u32::from_le_bytes(buf))
    }

    fn deserialize_u64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0x00u8; 8];
        TlvReader::read_slice(&mut self.reader, &mut buf)?;
        visitor.visit_u64(u64::from_le_bytes(buf))
    }

    fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0x00u8; 4];
        TlvReader::read_slice(&mut self.reader, &mut buf)?;
        visitor.visit_f32(f32::from_le_bytes(buf))
    }

    fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0x00u8; 8];
        TlvReader::read_slice(&mut self.reader, &mut buf)?;
        visitor.visit_f64(f64::from_le_bytes(buf))
    }

    fn deserialize_char<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0x00u8; 1];
        TlvReader::read_slice(&mut self.reader, &mut buf)?;
        visitor.visit_char(char::from(buf[0]))
    }

    fn deserialize_str<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unsafe {
            self.next()
                .unwrap_or(Err(Error::Io(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "It was not possible to deserialize the text",
                ))))
                .and_then(|bytes| {
                    visitor.visit_str(std::str::from_utf8_unchecked(bytes.as_slice()))
                })
        }
    }

    fn deserialize_string<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unsafe {
            self.next()
                .unwrap_or(Err(Error::Io(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "It was not possible to deserialize the text",
                ))))
                .and_then(|bytes| visitor.visit_string(String::from_utf8_unchecked(bytes)))
        }
    }

    fn deserialize_bytes<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.next()
            .unwrap_or(Err(Error::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "It was not possible to deserialize the text",
            ))))
            .and_then(|bytes| visitor.visit_bytes(bytes.as_slice()))
    }

    fn deserialize_byte_buf<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.next()
            .unwrap_or(Err(Error::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "It was not possible to deserialize the text",
            ))))
            .and_then(|bytes| visitor.visit_byte_buf(bytes))
    }

    fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.next()
            .unwrap_or(Err(Error::Io(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "It was not possible to deserialize the text",
            ))))
            .and_then(|bytes| {
                if !bytes.is_empty() {
                    visitor.visit_byte_buf(bytes)
                } else {
                    visitor.visit_none()
                }
            })
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let buf = self.next().ok_or(Error::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes to read the list from the TLV format",
        )))??;

        let buf = TlvReader::new(buf.as_slice());

        visitor.visit_seq(buf)
    }

    fn deserialize_tuple<V>(mut self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let buf = self.next().ok_or(Error::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes to read the list from the TLV format",
        )))??;

        let buf = TlvReader::new(buf.as_slice());

        visitor.visit_seq(buf)
    }

    fn deserialize_tuple_struct<V>(
        mut self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let buf = self.next().ok_or(Error::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes to read the list from the TLV format",
        )))??;

        let buf = TlvReader::new(buf.as_slice());

        visitor.visit_seq(buf)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        mut self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let buf = self.next().ok_or(Error::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes to read the list from the TLV format",
        )))??;

        let buf = TlvReader::new(buf.as_slice());

        visitor.visit_seq(buf)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use serde::de::Deserialize;
    use serde::ser::Serialize;
    use std::io::{Cursor, Write};
    use std::iter;

    #[test]
    fn tlv_reader_vec() {
        let buf: Vec<u8> = iter::repeat(())
            .take(65536)
            .enumerate()
            .map(|(i, _)| i as u8)
            .collect();

        let buf_other: Vec<u8> = iter::repeat(())
            .take(10)
            .enumerate()
            .map(|(i, _)| i as u8)
            .collect();

        let cursor = Cursor::new(Vec::<u8>::new());
        let mut tlv_writer = TlvWriter::new(cursor);

        tlv_writer.write(buf.as_slice()).unwrap();
        tlv_writer.write(buf_other.as_slice()).unwrap();

        let mut cursor = tlv_writer.into_inner();
        cursor.set_position(0);

        let mut tlv_reader = TlvReader::new(cursor);

        let fetch_vec = tlv_reader.next().unwrap().unwrap();
        assert_eq!(buf, fetch_vec);

        let fetch_vec = tlv_reader.next().unwrap().unwrap();
        assert_eq!(buf_other, fetch_vec);
    }

    #[test]
    fn tlv_reader_deserialize_u64() {
        let cursor = Cursor::new(Vec::<u8>::new());

        let input = 2533u64;

        let mut tlv_writer = TlvWriter::new(cursor);
        input.serialize(&mut tlv_writer).unwrap();

        let mut cursor = tlv_writer.into_inner();
        cursor.set_position(0);

        let tlv_reader = TlvReader::new(cursor);
        let output = Deserialize::deserialize(tlv_reader).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn tlv_reader_list() {
        let cursor = Cursor::new(Vec::<u8>::new());

        let input = vec![2558usize, 21, 37, 2009];
        let mut tlv_writer = TlvWriter::new(cursor);

        let list = input
            .iter()
            .map(|i| i.to_le_bytes())
            .collect::<Vec<[u8; 8]>>();
        tlv_writer.write_list(list.as_slice()).unwrap();

        let mut cursor = tlv_writer.into_inner();
        cursor.set_position(0);

        let mut tlv_reader = TlvReader::new(cursor);
        let output: Vec<usize> = tlv_reader
            .read_list::<Vec<u8>>()
            .unwrap()
            .iter()
            .map(|i| {
                let mut n = [0x00u8; 8];
                n.copy_from_slice(i.as_slice());
                usize::from_le_bytes(n)
            })
            .collect();

        assert_eq!(input, output);
    }
}
