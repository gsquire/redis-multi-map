package main

import (
	"math/rand"
	"time"

	"github.com/tidwall/redbench"
)

var letterRunes = []rune("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")

func init() {
	rand.Seed(time.Now().UnixNano())
}

func randomString(length int) string {
	b := make([]rune, length)
	for i := range b {
		b[i] = letterRunes[rand.Intn(len(letterRunes))]
	}

	return string(b)
}

func main() {
	redbench.Bench("MULTIMAP.INSERT", "127.0.0.1:6379", nil, nil, func(buf []byte) []byte {
		return redbench.AppendCommand(buf, "MULTIMAP.INSERT", "map", randomString(5), randomString(10))
	})

	redbench.Bench("SET", "127.0.0.1:6379", nil, nil, func(buf []byte) []byte {
		return redbench.AppendCommand(buf, "SET", randomString(5), randomString(10))
	})
}
