package thaibreak

import (
	"os"
	"path/filepath"
	"strings"
	"sync"
)

var (
	defaultMu          sync.RWMutex
	defaultTokenizer   *Tokenizer
	defaultLineBreaker *LineBreaker
	defaultInitOnce    sync.Once
)

func initDefault() {
	defaultInitOnce.Do(func() {
		// Look for dictionary in common locations
		candidates := []string{
			"../data/words.txt",
			"data/words.txt",
			"../../data/words.txt",
		}
		var dictPath, bigramPath string
		for _, c := range candidates {
			if _, err := os.Stat(c); err == nil {
				dictPath = c
				dir := filepath.Dir(c)
				b := filepath.Join(dir, "bigrams.tsv")
				if _, err2 := os.Stat(b); err2 == nil {
					bigramPath = b
				}
				break
			}
		}

		trie := NewThaiTrie()
		if dictPath != "" {
			if t, err := LoadTsvFile(dictPath); err == nil {
				trie = t
			}
		}

		var bigrams *BigramModel
		if bigramPath != "" {
			if b, err := LoadBigramsFile(bigramPath); err == nil {
				bigrams = b
			}
		}

		defaultTokenizer = NewTokenizer(trie, bigrams)
		defaultLineBreaker = NewLineBreaker(defaultTokenizer)
	})
}

// SetDefault configures the global shared tokenizer instance.
func SetDefault(tok *Tokenizer) {
	defaultMu.Lock()
	defer defaultMu.Unlock()
	defaultTokenizer = tok
	defaultLineBreaker = NewLineBreaker(tok)
}

// GetDefaultTokenizer returns the global shared Tokenizer.
func GetDefaultTokenizer() *Tokenizer {
	initDefault()
	defaultMu.RLock()
	defer defaultMu.RUnlock()
	return defaultTokenizer
}

// GetDefaultLineBreaker returns the global shared LineBreaker.
func GetDefaultLineBreaker() *LineBreaker {
	initDefault()
	defaultMu.RLock()
	defer defaultMu.RUnlock()
	return defaultLineBreaker
}

// Words tokenizes Thai text into words.
func Words(text string) []string {
	return GetDefaultTokenizer().Tokenize(text, false)
}

// Tokenize is an alias of Words.
func Tokenize(text string) []string {
	return Words(text)
}

// Join tokenizes and joins tokens with a delimiter.
func Join(text string, sep string) string {
	return strings.Join(Words(text), sep)
}


// Lines inserts Zero-Width Space (U+200B) break markers adhering to Thai typographic rules.
func Lines(text string, isHtml bool) string {
	return GetDefaultLineBreaker().InsertLineBreaks(text, DefaultBreakMarker, isHtml)
}

// BreakLines is an alias of Lines.
func BreakLines(text string, isHtml bool) string {
	return Lines(text, isHtml)
}

// Wrap soft-wraps Thai text to fit within `width` visual columns.
func Wrap(text string, width int) string {
	return GetDefaultLineBreaker().Wrap(text, width, "\n", false)
}

// DisplayWidth returns the visual column width for Thai text.
func DisplayWidth(text string) int {
	return ThaiDisplayWidth(text)
}
