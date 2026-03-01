package vectorizer

import "testing"

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
	if v.Dimension != 32 {
		t.Fatalf("expected default dimension 32, got %d", v.Dimension)
	}
}
