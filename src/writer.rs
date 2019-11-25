use crate::Error;

use std::io::{self, Write};

use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::Serialize;

macro_rules! implemented_ser_trait_unimplemented {
    ($t:ty,$m:ident) => {
        impl<'a, W> $t for &'a mut TlvWriter<W>
        where
            W: io::Write,
        {
            type Ok = ();
            type Error = Error;

            fn $m<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: Serialize,
            {
                let r: &mut TlvWriter<W> = *self;
                value.serialize(r)
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }
        }
    };
}

macro_rules! implemented_ser_trait_unimplemented_two {
    ($t:ty,$m:ident,$m2:ident) => {
        impl<'a, W> $t for &'a mut TlvWriter<W>
        where
            W: io::Write,
        {
            type Ok = ();
            type Error = Error;

            fn $m<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
            where
                T: Serialize,
            {
                unimplemented!()
            }

            fn $m2<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
            where
                T: Serialize,
            {
                unimplemented!()
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                unimplemented!()
            }
        }
    };
}

macro_rules! implemented_ser_trait_unimplemented_field {
    ($t:ty,$m:ident) => {
        impl<'a, W> $t for &'a mut TlvWriter<W>
        where
            W: io::Write,
        {
            type Ok = ();
            type Error = Error;

            fn $m<T: ?Sized>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error>
            where
                T: Serialize,
            {
                unimplemented!()
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                unimplemented!()
            }
        }
    };
}

/// Optionally consumes an implementation of [`io::Write`], and provides an adapter to convert
/// slices of bytes to TLV format, and output the result to the writer.
pub struct TlvWriter<W>
where
    W: io::Write,
{
    writer: W,
}

impl<W> TlvWriter<W>
where
    W: io::Write,
{
    /// [`TlvWriter`] constructor
    pub fn new(writer: W) -> Self {
        TlvWriter { writer }
    }

    /// Consumes self, and return the inner writer
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Convert the provided slice of bytes to TLV format, and write the TL to the writer
    pub fn bytes_len_to_writer(writer: W, mut len: usize) -> Result<usize, Error> {
        let mut writer = writer;

        // If the buffer is empty, the type should be 0xf0
        if len == 0 {
            writer.write(&[0xf0])?;
            return Ok(0);
        }

        // The TLV length will be little-endian format
        let buf_len = len.to_le_bytes();

        // The TLV type will be 0xf`x`, where `x` is the amount of bytes that will be used by the
        // length.
        //
        // Therefore, `len_mask` will hold this amount of bytes. The bitwise operators will allow
        // a fast definition for that.
        let mut len_mask = 0x01;
        len >>= 8;
        while len > 0 {
            len_mask <<= 1;
            len >>= 8;
        }

        writer.write(&[0xf0 | len_mask])?;
        Ok(writer.write(&buf_len[..len_mask as usize])?)
    }

    /// Convert the provided slice of bytes to TLV format, output the result to the provided
    /// writer, and return the amount of bytes written.
    pub fn bytes_to_writer(mut writer: W, buf: &[u8]) -> Result<usize, Error> {
        TlvWriter::bytes_len_to_writer(&mut writer, buf.len())?;
        Ok(writer.write(buf)?)
    }

    /// Append the provided usize to the writer in TLV format
    pub fn write_usize(&mut self, n: usize) -> Result<usize, Error> {
        let n = n.to_le_bytes();

        TlvWriter::bytes_to_writer(&mut self.writer, &n[..])
    }

    /// Write a list of serializable items
    pub fn write_list<L: AsRef<[u8]>>(&mut self, list: &[L]) -> Result<usize, Error> {
        let buf: Vec<u8> = vec![];
        let mut writer = TlvWriter::new(buf);

        for item in list {
            writer.write(item.as_ref())?;
        }

        let buf = writer.into_inner();
        Ok(self.write(buf.as_slice())?)
    }
}

impl<W> io::Write for TlvWriter<W>
where
    W: io::Write,
{
    /// The [`io::Write::write`] implementation will internally call the
    /// [`TlvWriter::bytes_to_writer`]. Therefore, the provided buffer will first be converted to
    /// TLV format, and then sent to the inner writer.
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        TlvWriter::bytes_to_writer(&mut self.writer, buf).map_err(|e| e.into())
    }

    /// The [`io::Write::flush`] implementation will just forward the call to the inner writer.
    fn flush(&mut self) -> Result<(), io::Error> {
        self.writer.flush()
    }
}

impl<'a, W> SerializeSeq for &mut TlvWriter<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let r: &mut TlvWriter<W> = *self;
        value.serialize(r)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

implemented_ser_trait_unimplemented!(SerializeTuple, serialize_element);
implemented_ser_trait_unimplemented!(SerializeTupleStruct, serialize_field);
implemented_ser_trait_unimplemented!(SerializeTupleVariant, serialize_field);
implemented_ser_trait_unimplemented_two!(SerializeMap, serialize_key, serialize_value);
implemented_ser_trait_unimplemented_field!(SerializeStruct, serialize_field);
implemented_ser_trait_unimplemented_field!(SerializeStructVariant, serialize_field);

impl<'a, W> serde::Serializer for &'a mut TlvWriter<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&[if v { 0x01u8 } else { 0x00u8 }]).map(|_| ())?)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&v.to_le_bytes()[..]).map(|_| ())?)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&v.to_le_bytes()[..]).map(|_| ())?)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&v.to_le_bytes()[..]).map(|_| ())?)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&v.to_le_bytes()[..]).map(|_| ())?)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&v.to_le_bytes()[..]).map(|_| ())?)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&v.to_le_bytes()[..]).map(|_| ())?)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&v.to_le_bytes()[..]).map(|_| ())?)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&v.to_le_bytes()[..]).map(|_| ())?)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&v.to_le_bytes()[..]).map(|_| ())?)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&v.to_le_bytes()[..]).map(|_| ())?)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&[v as u8]).map(|_| ())?)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(v.as_bytes()).map(|_| ())?)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(v).map(|_| ())?)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&[]).map(|_| ())?)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&[]).map(|_| ())?)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&[]).map(|_| ())?)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(self.write(&[]).map(|_| ())?)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.write(&variant_index.to_le_bytes()[..]).map(|_| ())?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len = len.ok_or(Error::Io(io::Error::new(
            io::ErrorKind::Other,
            "The leading size is mandatory",
        )))?;

        TlvWriter::bytes_len_to_writer(&mut self.writer, len)?;
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        TlvWriter::bytes_len_to_writer(&mut self.writer, len)?;
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        TlvWriter::bytes_len_to_writer(&mut self.writer, len)?;
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        TlvWriter::bytes_len_to_writer(&mut self.writer, len)?;
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let len = len.ok_or(Error::Io(io::Error::new(
            io::ErrorKind::Other,
            "The leading size is mandatory",
        )))?;

        TlvWriter::bytes_len_to_writer(&mut self.writer, len)?;
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        TlvWriter::bytes_len_to_writer(&mut self.writer, len)?;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        TlvWriter::bytes_len_to_writer(&mut self.writer, len)?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::io::{Cursor, Write};
    use std::iter;

    #[test]
    fn tlv_writer() {
        let buf: Vec<u8> = iter::repeat(())
            .take(25)
            .enumerate()
            .map(|(i, _)| i as u8)
            .collect();

        let cursor = Cursor::new(Vec::<u8>::new());
        let mut tlv = TlvWriter::new(cursor);
        tlv.write(buf.as_slice()).unwrap();

        let cursor = tlv.into_inner();
        let result = cursor.into_inner();

        assert_eq!(0xf1u8, result[0]);
        assert_eq!(0x19u8, result[1]);
        assert_eq!(buf.as_slice(), &result[2..]);
    }

    #[test]
    fn tlv_writer_zero() {
        let cursor = Cursor::new(Vec::<u8>::new());
        let mut tlv = TlvWriter::new(cursor);
        tlv.write(&[][..]).unwrap();

        let cursor = tlv.into_inner();
        let result = cursor.into_inner();

        assert_eq!(0xf0u8, result[0]);
    }

    #[test]
    fn tlv_writer_one() {
        let buf: Vec<u8> = iter::repeat(())
            .take(1)
            .enumerate()
            .map(|(i, _)| i as u8)
            .collect();

        let cursor = Cursor::new(Vec::<u8>::new());
        let mut tlv = TlvWriter::new(cursor);
        tlv.write(buf.as_slice()).unwrap();

        let cursor = tlv.into_inner();
        let result = cursor.into_inner();

        assert_eq!(0xf1u8, result[0]);
        assert_eq!(0x01u8, result[1]);
        assert_eq!(buf.as_slice(), &result[2..]);
    }

    #[test]
    fn tlv_writer_many() {
        let buf: Vec<u8> = iter::repeat(())
            .take(65536)
            .enumerate()
            .map(|(i, _)| i as u8)
            .collect();

        let cursor = Cursor::new(Vec::<u8>::new());
        let mut tlv = TlvWriter::new(cursor);
        tlv.write(buf.as_slice()).unwrap();

        let cursor = tlv.into_inner();
        let result = cursor.into_inner();

        assert_eq!(0xf4u8, result[0]);
        assert_eq!(&[0x00u8, 0x00, 0x01, 0x00], &result[1..5]);
        assert_eq!(buf.as_slice(), &result[5..]);
    }
}
