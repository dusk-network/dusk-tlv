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
