package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"strings"

	"text2vec/text2vec/internal/vectorizer"
)

func main() {
	dim := flag.Int("dim", 32, "vector dimension")
	flag.Parse()

	text := strings.TrimSpace(strings.Join(flag.Args(), " "))
	if text == "" {
		fmt.Fprintln(os.Stderr, "usage: text2vec [-dim N] <text>")
		os.Exit(1)
	}

	v := vectorizer.New(*dim)
	vec := v.Encode(text)

	enc := json.NewEncoder(os.Stdout)
	enc.SetIndent("", "  ")
	if err := enc.Encode(vec); err != nil {
		fmt.Fprintln(os.Stderr, "encode error:", err)
		os.Exit(1)
	}
}
