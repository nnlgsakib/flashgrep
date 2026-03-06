package main

import (
	"bufio"
	"encoding/json"
	"flag"
	"fmt"
	"io"
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
	tui := flag.Bool("tui", false, "start an interactive terminal UI")
	flag.Parse()

	v := vectorizer.NewWithConfig(vectorizer.Config{
		Dimension:     *dim,
		MaxNGram:      *maxNGram,
		UseSignedHash: *signedHash,
		SublinearTF:   *sublinearTF,
		Normalize:     *normalize,
	})

	if *tui {
		runTUI(v)
		return
	}

	text := strings.TrimSpace(strings.Join(flag.Args(), " "))
	if text == "" {
		fmt.Fprintln(os.Stderr, "usage: text2vec [flags] <text>")
		flag.PrintDefaults()
		os.Exit(1)
	}

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

func runTUI(v vectorizer.Vectorizer) {
	reader := bufio.NewReader(os.Stdin)

	fmt.Println("text2vec interactive mode")
	fmt.Println("Type text and press Enter to vectorize.")
	fmt.Println("Commands: :q to quit, :config to print active config, Enter on compare prompt to skip.")
	fmt.Println()

	for {
		input, ok := prompt(reader, "text> ")
		if !ok {
			fmt.Println()
			return
		}

		input = strings.TrimSpace(input)
		switch input {
		case "":
			continue
		case ":q", ":quit", "exit":
			return
		case ":config":
			cfg := v.Config()
			fmt.Printf("config: dimension=%d max-ngram=%d signed-hash=%t sublinear-tf=%t normalize=%t\n\n", cfg.Dimension, cfg.MaxNGram, cfg.UseSignedHash, cfg.SublinearTF, cfg.Normalize)
			continue
		}

		vec := v.Encode(input)
		fmt.Printf("vector: %s\n", formatVectorPreview(vec, 12))

		compare, ok := prompt(reader, "compare> ")
		if !ok {
			fmt.Println()
			return
		}

		compare = strings.TrimSpace(compare)
		switch compare {
		case "", ":skip":
			fmt.Println()
			continue
		case ":q", ":quit", "exit":
			return
		case ":config":
			cfg := v.Config()
			fmt.Printf("config: dimension=%d max-ngram=%d signed-hash=%t sublinear-tf=%t normalize=%t\n\n", cfg.Dimension, cfg.MaxNGram, cfg.UseSignedHash, cfg.SublinearTF, cfg.Normalize)
			continue
		}

		similarity := v.CosineSimilarity(input, compare)
		fmt.Printf("cosine similarity: %.6f\n\n", similarity)
	}
}

func prompt(reader *bufio.Reader, label string) (string, bool) {
	fmt.Print(label)
	line, err := reader.ReadString('\n')
	if err != nil {
		if err == io.EOF {
			if len(line) == 0 {
				return "", false
			}
			return line, true
		}
		fmt.Fprintln(os.Stderr, "read error:", err)
		return "", false
	}
	return line, true
}

func formatVectorPreview(vec []float64, previewSize int) string {
	if previewSize <= 0 || len(vec) <= previewSize {
		return fmt.Sprintf("%v", vec)
	}

	preview := make([]string, 0, previewSize)
	for i := 0; i < previewSize; i++ {
		preview = append(preview, fmt.Sprintf("%.4f", vec[i]))
	}
	return fmt.Sprintf("[%s, ... (%d values)]", strings.Join(preview, ", "), len(vec))
}
