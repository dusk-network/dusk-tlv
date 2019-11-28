package tlv

import (
	"bytes"
	"log"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestTlvReaderToBytes(t *testing.T) {
	buf := make([]byte, 65536)
	rand.Read(buf)

	bb := bytes.NewBuffer([]byte{})
	tl := NewWriter(bb)
	_, err := tl.Write(buf)
	if err != nil {
		log.Fatal(err)
	}

	fetch, err := ReaderToBytes(bb)
	if err != nil {
		log.Fatal(err)
	}

	assert.Equal(t, buf, fetch)
}

func TestTlvRead(t *testing.T) {
	buf := make([]byte, 2048)
	rand.Read(buf)

	noise := make([]byte, 2048)
	rand.Read(buf)

	bb := bytes.NewBuffer([]byte{})
	tl := NewWriter(bb)
	_, err := tl.Write(buf)
	if err != nil {
		log.Fatal(err)
	}
	_, err = tl.Write(noise)
	if err != nil {
		log.Fatal(err)
	}

	fetch := make([]byte, 2048)
	_, err = Read(bb, fetch)
	if err != nil {
		log.Fatal(err)
	}

	assert.Equal(t, buf, fetch)
}

func TestTlvReaderToList(t *testing.T) {
	list := [][]uint8{[]byte{0x15, 0xff}, []byte{0x20}, []byte{0x30}, []byte{0x40}}

	bb := bytes.NewBuffer([]byte{})
	tl := NewWriter(bb)
	_, err := tl.WriteList(list)
	if err != nil {
		log.Fatal(err)
	}

	fetch, err := ReaderToList(bb)
	if err != nil {
		log.Fatal(err)
	}

	assert.Equal(t, list, fetch)
}
