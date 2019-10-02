# Dusk TLV

Rust implementation for TLV encoding scheme.

## Structure

1) Type
- Length: Fixed, 1 byte
- Contents: 0xf`x`, where `x` is the amount of bytes that will compose the length

2) Length
- Length: Variable, defined by `1) Type`
- Contents: Little-endian order amount of bytes that defines the value length

3) Value
- Length: Variable, define by `2) Length`
- Contents: Slice of bytes of fixed size

## Example
```rust
use dusk_tlv::{TlvReader, TlvWriter};
use std::io::Write;

let mut writer = TlvWriter::new(vec![]);
writer.write(b"Hello World!").unwrap();
writer.write(b"Foo").unwrap();
writer.write(b"Bar").unwrap();

let v = writer.into_inner();

let tlv_hello_world = &v[0..14];
let tlv_foo = &v[14..19];
let tlv_bar = &v[19..24];

// Type 0xf1, for the length is 1 byte
assert_eq!(0xf1, tlv_hello_world[0]);
// Length 0x0c, for the payload is 12 bytes
assert_eq!(0x0c, tlv_hello_world[1]);
// Payload
assert_eq!(b"Hello World!", &tlv_hello_world[2..]);

assert_eq!(0xf1, tlv_foo[0]);
assert_eq!(0x03, tlv_foo[1]);
assert_eq!(b"Foo", &tlv_foo[2..]);

assert_eq!(0xf1, tlv_bar[0]);
assert_eq!(0x03, tlv_bar[1]);
assert_eq!(b"Bar", &tlv_bar[2..]);

let mut s = v.as_slice();
let mut reader = TlvReader::new(&mut s);
assert_eq!(b"Hello World!", reader.next().unwrap().unwrap().as_slice());
assert_eq!(b"Foo", reader.next().unwrap().unwrap().as_slice());
assert_eq!(b"Bar", reader.next().unwrap().unwrap().as_slice());
assert!(reader.next().is_none());
```
