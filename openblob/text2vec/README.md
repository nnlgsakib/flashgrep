# text2vec

Small Go example that converts text into a fixed-size numeric vector.

## How it works

- Lowercases input text.
- Splits text into alphanumeric tokens.
- Hashes each token into one of `N` buckets (vector dimensions).
- Counts token hits per bucket.
- Applies L2 normalization so vector length is 1.

This is a lightweight hash-based embedding approach useful for demos and small pipelines.

## Run

```bash
go run ./text2vec/cmd/text2vec -dim 16 "hello world hello"
```

## Test

```bash
go test ./...
```
