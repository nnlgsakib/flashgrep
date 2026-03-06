package vectorizer

import (
	"math"
	"testing"
)

func TestEncodeDeterministic(t *testing.T) {
	v := New(16)
	a := v.Encode("Hello world hello")
	b := v.Encode("Hello world hello")

	if len(a) != 16 || len(b) != 16 {
		t.Fatalf("unexpected vector size: %d %d", len(a), len(b))
	}

	for i := range a {
		if a[i] != b[i] {
			t.Fatalf("vector not deterministic at index %d: %f != %f", i, a[i], b[i])
		}
	}
}

func TestEncodeEmptyText(t *testing.T) {
	v := New(8)
	vec := v.Encode("")
	if len(vec) != 8 {
		t.Fatalf("unexpected vector size: %d", len(vec))
	}
	for i, x := range vec {
		if x != 0 {
			t.Fatalf("expected zeros for empty input, found %f at %d", x, i)
		}
	}
}

func TestNewDefaultDimension(t *testing.T) {
	v := New(0)
	if v.Config().Dimension != 32 {
		t.Fatalf("expected default dimension 32, got %d", v.Config().Dimension)
	}
}

func TestCosineSimilaritySameText(t *testing.T) {
	v := NewWithConfig(Config{Dimension: 64, MaxNGram: 2, Normalize: true, UseSignedHash: true, SublinearTF: true})
	sim := v.CosineSimilarity("hashing makes vectors", "hashing makes vectors")
	if math.Abs(sim-1.0) > 1e-9 {
		t.Fatalf("expected cosine similarity ~1.0, got %f", sim)
	}
}

func TestMaxNGramChangesRepresentation(t *testing.T) {
	base := NewWithConfig(Config{Dimension: 1024, MaxNGram: 1, Normalize: false, UseSignedHash: false, SublinearTF: false})
	phrases := NewWithConfig(Config{Dimension: 1024, MaxNGram: 2, Normalize: false, UseSignedHash: false, SublinearTF: false})

	a := base.Encode("new york city life")
	b := phrases.Encode("new york city life")

	if len(a) != len(b) {
		t.Fatalf("vector dimensions differ: %d vs %d", len(a), len(b))
	}

	for i := range a {
		if a[i] != b[i] {
			return
		}
	}
	t.Fatalf("expected n-gram configuration to change vector representation")
}

func TestSignedHashProducesNegativeOrPositiveContributions(t *testing.T) {
	idx, sign := hashToIndexAndSign("signed_hash_probe", 32, true)
	if idx < 0 || idx >= 32 {
		t.Fatalf("index out of bounds: %d", idx)
	}
	if sign != -1.0 && sign != 1.0 {
		t.Fatalf("unexpected sign: %f", sign)
	}
}

func TestSublinearTFReducesWeightGrowth(t *testing.T) {
	plain := NewWithConfig(Config{Dimension: 64, MaxNGram: 1, Normalize: false, UseSignedHash: false, SublinearTF: false})
	scaled := NewWithConfig(Config{Dimension: 64, MaxNGram: 1, Normalize: false, UseSignedHash: false, SublinearTF: true})

	repeated := "echo echo echo echo echo"
	plainVec := plain.Encode(repeated)
	scaledVec := scaled.Encode(repeated)
	plainNorm := math.Sqrt(dot(plainVec, plainVec))
	scaledNorm := math.Sqrt(dot(scaledVec, scaledVec))

	if scaledNorm >= plainNorm {
		t.Fatalf("expected sublinear tf norm (%f) to be smaller than plain tf norm (%f)", scaledNorm, plainNorm)
	}
}
