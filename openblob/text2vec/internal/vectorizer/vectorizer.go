package vectorizer

import (
	"hash/fnv"
	"math"
	"strings"
	"unicode"
)

// Config controls vectorization behavior.
type Config struct {
	Dimension     int  `json:"dimension"`
	MaxNGram      int  `json:"max_ngram"`
	UseSignedHash bool `json:"use_signed_hash"`
	SublinearTF   bool `json:"sublinear_tf"`
	Normalize     bool `json:"normalize"`
}

// Vectorizer converts text into fixed-size vectors.
type Vectorizer struct {
	cfg Config
}

// DefaultConfig returns a balanced default setup for small pipelines.
func DefaultConfig() Config {
	return Config{
		Dimension:     32,
		MaxNGram:      1,
		UseSignedHash: true,
		SublinearTF:   true,
		Normalize:     true,
	}
}

// New keeps backward compatibility with the original constructor.
func New(dimension int) Vectorizer {
	cfg := DefaultConfig()
	if dimension > 0 {
		cfg.Dimension = dimension
	}
	return NewWithConfig(cfg)
}

// NewWithConfig creates a vectorizer with advanced options.
func NewWithConfig(cfg Config) Vectorizer {
	defaults := DefaultConfig()
	if cfg.Dimension <= 0 {
		cfg.Dimension = defaults.Dimension
	}
	if cfg.MaxNGram <= 0 {
		cfg.MaxNGram = defaults.MaxNGram
	}
	return Vectorizer{cfg: cfg}
}

// Config returns the active configuration.
func (v Vectorizer) Config() Config {
	return v.cfg
}

func (v Vectorizer) Encode(text string) []float64 {
	vec := make([]float64, v.cfg.Dimension)
	tokens := tokenize(text)
	features := buildNGrams(tokens, v.cfg.MaxNGram)
	if len(features) == 0 {
		return vec
	}

	counts := make(map[string]float64, len(features))
	for _, feature := range features {
		counts[feature] += 1
	}

	for feature, tf := range counts {
		weight := tf
		if v.cfg.SublinearTF {
			weight = 1 + math.Log(tf)
		}
		idx, sign := hashToIndexAndSign(feature, v.cfg.Dimension, v.cfg.UseSignedHash)
		vec[idx] += sign * weight
	}

	if v.cfg.Normalize {
		normalizeL2(vec)
	}
	return vec
}

// CosineSimilarity compares two texts in the configured vector space.
func (v Vectorizer) CosineSimilarity(a, b string) float64 {
	va := v.Encode(a)
	vb := v.Encode(b)

	dotAB := dot(va, vb)
	if v.cfg.Normalize {
		return dotAB
	}

	normA := math.Sqrt(dot(va, va))
	normB := math.Sqrt(dot(vb, vb))
	if normA == 0 || normB == 0 {
		return 0
	}
	return dotAB / (normA * normB)
}

func tokenize(text string) []string {
	lower := strings.ToLower(text)
	parts := strings.FieldsFunc(lower, func(r rune) bool {
		return !unicode.IsLetter(r) && !unicode.IsNumber(r)
	})
	return parts
}

func buildNGrams(tokens []string, maxNGram int) []string {
	if len(tokens) == 0 {
		return nil
	}
	if maxNGram < 1 {
		maxNGram = 1
	}

	features := make([]string, 0, len(tokens)*maxNGram)
	for n := 1; n <= maxNGram; n++ {
		for i := 0; i+n <= len(tokens); i++ {
			if n == 1 {
				features = append(features, tokens[i])
				continue
			}
			features = append(features, strings.Join(tokens[i:i+n], "_"))
		}
	}
	return features
}

func hashToIndexAndSign(feature string, dim int, useSignedHash bool) (int, float64) {
	h := fnv.New64a()
	_, _ = h.Write([]byte(feature))
	sum := h.Sum64()

	idx := int(sum % uint64(dim))
	sign := 1.0
	if useSignedHash && (sum&1) == 1 {
		sign = -1.0
	}
	return idx, sign
}

func dot(a, b []float64) float64 {
	limit := len(a)
	if len(b) < limit {
		limit = len(b)
	}
	var total float64
	for i := 0; i < limit; i++ {
		total += a[i] * b[i]
	}
	return total
}

func normalizeL2(vec []float64) {
	norm := math.Sqrt(dot(vec, vec))
	if norm == 0 {
		return
	}
	invNorm := 1.0 / norm
	for i := range vec {
		vec[i] *= invNorm
	}
}
