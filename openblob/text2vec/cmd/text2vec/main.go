package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"strings"

	"text2vec/text2vec/internal/vectorizer"
)

type output struct {
	Vector     []float64         `json:"vector"`
	Similarity *float64          `json:"similarity,omitempty"`
	Config     vectorizer.Config `json:"config"`
}

func main() {
	dim := flag.Int("dim", 32, "vector dimension")
	maxNGram := flag.Int("max-ngram", 1, "maximum n-gram size (1=unigrams, 2=add bigrams, etc)")
	signedHash := flag.Bool("signed-hash", true, "use signed feature hashing to reduce collision bias")
	sublinearTF := flag.Bool("sublinear-tf", true, "use 1+log(tf) term-frequency weighting")
	normalize := flag.Bool("normalize", true, "apply L2 normalization")
	compareText := flag.String("compare", "", "optional second text for cosine similarity")
	flag.Parse()

	text := strings.TrimSpace(strings.Join(flag.Args(), " "))
	if text == "" {
		fmt.Fprintln(os.Stderr, "usage: text2vec [flags] <text>")
		flag.PrintDefaults()
		os.Exit(1)
	}

	v := vectorizer.NewWithConfig(vectorizer.Config{
		Dimension:     *dim,
		MaxNGram:      *maxNGram,
		UseSignedHash: *signedHash,
		SublinearTF:   *sublinearTF,
		Normalize:     *normalize,
	})

	result := output{
		Vector: v.Encode(text),
		Config: v.Config(),
	}
	if strings.TrimSpace(*compareText) != "" {
		score := v.CosineSimilarity(text, *compareText)
		result.Similarity = &score
	}

	enc := json.NewEncoder(os.Stdout)
	enc.SetIndent("", "  ")
	if err := enc.Encode(result); err != nil {
		fmt.Fprintln(os.Stderr, "encode error:", err)
		os.Exit(1)
	}
}
