package tlv

import (
	"encoding/binary"
	"io"
)

// Writer implements Type-Length-Value encoding scheme
type Writer struct {
	w io.Writer
}

// NewWriter returns a new instance of a TLV Writer. The instance will have an inner implementation of io.Writer and will output the bytes to it.
func NewWriter(w io.Writer) Writer {
	return Writer{w: w}
}

// Write will encode the provided bytes into TLV format, and then write the result to the inner writer.
func (t *Writer) Write(p []byte) (int, error) {
	// Transform platform dependent int to uint64
	l := uint64(len(p))

	// Store the length of p into tlvL as little endian representation
	tlvL := make([]byte, 8)
	binary.LittleEndian.PutUint64(tlvL[:], l)

	// Set m as the required number of bytes to represent l
	m := 0x01
	l = l >> 8
	for ; l > 0; l = l >> 8 {
		m = m << 1
	}

	// Apply the var length mask and write the type
	tlvF := uint8(0xf0 | m)
	_, err := t.w.Write([]byte{tlvF})
	if err != nil {
		return 0, err
	}

	// Write the length
	_, err = t.w.Write(tlvL[:m])
	if err != nil {
		return 0, err
	}

	// Write the payload
	return t.w.Write(p)
}
