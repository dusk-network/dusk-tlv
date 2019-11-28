package tlv

import (
	"bytes"
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

// Write will call the inner Write function to output p in TLV format.
func (t *Writer) Write(p []byte) (int, error) {
	return Write(t.w, p)
}

// Write will encode the provided bytes into TLV format, and then write the result to the inner writer.
func Write(w io.Writer, p []byte) (int, error) {
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
	_, err := w.Write([]byte{tlvF})
	if err != nil {
		return 0, err
	}

	// Write the length
	_, err = w.Write(tlvL[:m])
	if err != nil {
		return 0, err
	}

	// Write the payload
	return w.Write(p)
}

// WriteList will call the inner WriteList function to output a list in TLV format.
func (t *Writer) WriteList(l [][]byte) (int, error) {
	return WriteList(t.w, l)
}

// WriteList will serialize a list in TLV format and output it to the inner writer
func WriteList(w io.Writer, l [][]byte) (int, error) {
	bytesBuffer := bytes.NewBuffer([]byte{})
	tlvWriter := NewWriter(bytesBuffer)
	n := 0

	for i := 0; i < len(l); i++ {
		ni, err := tlvWriter.Write(l[i])
		if err != nil {
			return 0, err
		}
		n += ni
	}

	return Write(w, bytesBuffer.Bytes())
}
