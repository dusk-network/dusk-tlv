package tlv

import (
	"bytes"
	"log"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestTlvWriterSnall(t *testing.T) {
	buf := make([]byte, 2)
	rand.Read(buf)

	bb := bytes.NewBuffer([]byte{})
	tl := NewWriter(bb)
	_, err := tl.Write(buf)
	if err != nil {
		log.Fatal(err)
	}

	assert.Equal(t, uint8(0xf1), bb.Bytes()[0])
	assert.Equal(t, uint8(0x02), bb.Bytes()[1])
	assert.Equal(t, buf, bb.Bytes()[2:])
}

func TestTlvWriterNormal(t *testing.T) {
	buf := make([]byte, 2500)
	rand.Read(buf)

	bb := bytes.NewBuffer([]byte{})
	tl := NewWriter(bb)
	_, err := tl.Write(buf)
	if err != nil {
		log.Fatal(err)
	}

	assert.Equal(t, uint8(0xf2), bb.Bytes()[0])
	assert.Equal(t, []byte{0xc4, 0x09}, bb.Bytes()[1:3])
	assert.Equal(t, buf, bb.Bytes()[3:])
}
