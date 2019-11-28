package tlv

import (
	"bytes"
	"encoding/binary"
	"io"
)

// ReadSize will fetch the size of the next TLV-formatted chunk from the provided reader.
func ReadSize(r io.Reader) (uint64, error) {
	// Get the type definition
	tlvT := make([]byte, 1)
	_, err := io.ReadFull(r, tlvT)
	if err != nil {
		return 0, err
	}

	// Extract the payload length from the type
	pl := uint8(0x0f) & tlvT[0]

	// Get the length of the payload from the reader
	tlvL := make([]byte, pl)
	_, err = io.ReadFull(r, tlvL)
	if err != nil {
		return 0, err
	}

	// Get the payload effective size from the tlv length
	ss := make([]byte, 8)
	copy(ss[:pl], tlvL[:pl])

	return binary.LittleEndian.Uint64(ss), nil
}

// ReaderToBytes will extract a TLV-formatted slice of bytes from an implementation of io.Read
func ReaderToBytes(r io.Reader) ([]byte, error) {
	s, err := ReadSize(r)
	if err != nil {
		return nil, err
	}

	// Fetch the buffer
	buf := make([]byte, s)
	_, err = io.ReadFull(r, buf)
	if err != nil {
		return nil, err
	}

	return buf, nil
}

// Read will attempt to read the TLV-formatted contents of the provided reader into buf. It will fail if the buf is not big enough.
func Read(r io.Reader, buf []byte) (int, error) {
	s, err := ReadSize(r)
	if err != nil {
		return 0, err
	}

	return io.ReadAtLeast(r, buf, int(s))
}

// ReaderToList will extract a TLV-formatted list from an implementation of io.Read
func ReaderToList(r io.Reader) ([][]byte, error) {
	list := make([][]byte, 0)

	buf, err := ReaderToBytes(r)
	if err != nil {
		return nil, err
	}

	bb := bytes.NewBuffer(buf)
	var b []byte
	for err == nil {
		b, err = ReaderToBytes(bb)
		if err == nil {
			list = append(list, b)
		}
	}

	return list, nil
}
