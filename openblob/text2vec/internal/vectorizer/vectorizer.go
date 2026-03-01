package vectorizer

import (
	"hash/fnv"
	"math"
	"strings"
	"unicode"
)

// Vectorizer converts text into fixed-size vectors.
type Vectorizer struct {
	Dimension int
}

func New(dimension int) Vectorizer {
	if dimension <= 0 {
		dimension = 32
	}
	return Vectorizer{Dimension: dimension}
}

func (v Vectorizer) Encode(text string) []float64 {
	vec := make([]float64, v.Dimension)
	tokens := tokenize(text)
	if len(tokens) == 0 {
		return vec
	}

	for _, token := range tokens {
		idx := hashToIndex(token, v.Dimension)
		vec[idx] += 1.0
	}

	normalizeL2(vec)
	return vec
}

func tokenize(text string) []string {
	lower := strings.ToLower(text)
	parts := strings.FieldsFunc(lower, func(r rune) bool {
		return !unicode.IsLetter(r) && !unicode.IsNumber(r)
	})
	return parts
}

func hashToIndex(token string, dim int) int {
	h := fnv.New32a()
	_, _ = h.Write([]byte(token))
	return int(h.Sum32() % uint32(dim))
}

func normalizeL2(vec []float64) {
	var sumSquares float64
	for _, x := range vec {
		sumSquares += x * x
	}
	if sumSquares == 0 {
		return
	}
	invNorm := 1.0 / math.Sqrt(sumSquares)
	for i := range vec {
		vec[i] *= invNorm
	}
}
