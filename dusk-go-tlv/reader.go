package tlv

import (
    "bytes"
	"encoding/binary"
	"io"
)

// ReaderToBytes will extract a TLV-formatted slice of bytes from an implementation of io.Read
func ReaderToBytes(r io.Reader) ([]byte, error) {
	// Get the type definition
	tlvT := make([]byte, 1)
	_, err := io.ReadFull(r, tlvT)
	if err != nil {
		return nil, err
	}

	// Extract the payload length from the type
	pl := uint8(0x0f) & tlvT[0]

	// Get the length of the payload from the reader
	tlvL := make([]byte, pl)
	_, err = io.ReadFull(r, tlvL)
	if err != nil {
		return nil, err
	}

	// Get the payload effective size from the tlv length
	ss := make([]byte, 8)
	copy(ss[:pl], tlvL[:pl])
	s := binary.LittleEndian.Uint64(ss)

	// Fetch the buffer
	buf := make([]byte, s)
	_, err = io.ReadFull(r, buf)
	if err != nil {
		return nil, err
	}

	return buf, nil
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
