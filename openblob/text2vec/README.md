# text2vec

Small Go utility that converts text into fixed-size numeric vectors using feature hashing.

## What is advanced now

- Configurable `n`-grams (`-max-ngram`) to capture phrase-level context.
- Signed feature hashing (`-signed-hash`) to reduce collision bias.
- Sublinear TF weighting (`-sublinear-tf`) with `1 + log(tf)`.
- Optional cosine similarity (`-compare`) for quick semantic checks.
- JSON output includes the active vectorizer config for reproducibility.

## How it works

1. Lowercases text.
2. Splits text into alphanumeric tokens.
3. Builds unigrams or higher-order n-grams.
4. Hashes features into one of `N` buckets.
5. Applies weighted accumulation (optionally signed).
6. Optionally L2-normalizes vectors.

## Run

```bash
go run ./text2vec/cmd/text2vec -dim 32 -max-ngram 2 "hello world hello"
```

## Similarity example

```bash
go run ./text2vec/cmd/text2vec -dim 64 -max-ngram 2 -compare "hello there world" "hello world"
```

## Test

```bash
go test ./...
```
