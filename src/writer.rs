use std::io;

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

    /// Convert the provided slice of bytes to TLV format, output the result to the provided
    /// writer, and return the amount of bytes written.
    pub fn bytes_to_writer(writer: W, buf: &[u8]) -> Result<usize, io::Error> {
        let mut writer = writer;
        let mut buf_len = buf.len();

        // If the buffer is empty, the type should be 0xf0
        if buf_len == 0 {
            writer.write(&[0xf0])?;
            return Ok(0);
        }

        // The TLV length will be little-endian format
        let len = buf_len.to_le_bytes();

        // The TLV type will be 0xf`x`, where `x` is the amount of bytes that will be used by the
        // length.
        //
        // Therefore, `len_mask` will hold this amount of bytes. The bitwise operators will allow
        // a fast definition for that.
        let mut len_mask = 0x01;
        buf_len >>= 8;
        while buf_len > 0 {
            len_mask <<= 1;
            buf_len >>= 8;
        }

        writer.write(&[0xf0 | len_mask])?;
        writer.write(&len[..len_mask as usize])?;
        writer.write(buf)
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
        TlvWriter::bytes_to_writer(&mut self.writer, buf)
    }

    /// The [`io::Write::flush`] implementation will just forward the call to the inner writer.
    fn flush(&mut self) -> Result<(), io::Error> {
        self.writer.flush()
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
