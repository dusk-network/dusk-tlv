# Dusk Go TLV

[![Build Status](https://travis-ci.com/dusk-network/dusk-go-tlv.svg?token=czzGwcZEd8hUsCLG3xJC&branch=master)](https://travis-ci.com/dusk-network/dusk-go-tlv)

GoLang implementation for TLV encoding scheme.


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

```go
package main

import (
	"bytes"
	"fmt"

	"github.com/dusk-network/dusk-tlv/dusk-go-tlv"
)

func main() {
	buf := []byte{0x15, 0x20}
	otherBuf := []byte{0xff, 0xfa, 0xfc}

	bytesBuffer := bytes.NewBuffer([]byte{})
	tlvWriter := tlv.NewWriter(bytesBuffer)

	// At this point, bytesBuffer.Bytes will be
	// []byte{0xf1, 0x02, 0x15, 0x20}
	tlvWriter.Write(buf)

	// At this point, bytesBuffer.Bytes will be
	// []byte{0xf1, 0x02, 0x15, 0x20, 0xf1, 0x03, 0xff, 0xfa, 0xfc}
	// that represents tlv(buf) + tlv(otherBuf)
	tlvWriter.Write(otherBuf)

	fetchedBytes, _ := tlv.ReaderToBytes(bytesBuffer)
	// Output: 0x1520
	fmt.Printf("%#x\n", fetchedBytes)

	fetchedBytes, _ = tlv.ReaderToBytes(bytesBuffer)
	// Output: 0xfffafc
	fmt.Printf("%#x\n", fetchedBytes)

	_, err := tlv.ReaderToBytes(bytesBuffer)
	// Expected error: EOF
	if err != nil {
		fmt.Println(err)
	}
}
```
