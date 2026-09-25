package thaibreak

import (
	"bufio"
	"io"
	"math"
	"os"
	"strconv"
	"strings"
	"sync"
)

// PrefixMatch represents a prefix match result.
type PrefixMatch struct {
	End    int     // Rune index where word ends (exclusive)
	Weight float64 // Word weight
}

// ThaiTrie implements a Flat Prefix Hash Map for fast word lookups.
type ThaiTrie struct {
	mu        sync.RWMutex
	prefixes  map[string]float64
	maxWeight float64
	// totalWeight is the sum of the weights of all full words (the unigram normalizer).
	totalWeight float64
}

// NewThaiTrie creates an empty ThaiTrie.
func NewThaiTrie() *ThaiTrie {
	return &ThaiTrie{
		prefixes:  make(map[string]float64),
		maxWeight: 1.0,
	}
}

// LoadTsv loads words and weights from a TSV reader.
func LoadTsv(r io.Reader) (*ThaiTrie, error) {
	trie := NewThaiTrie()
	scanner := bufio.NewScanner(r)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		parts := strings.Split(line, "\t")
		word := parts[0]
		weight := 1.0
		if len(parts) >= 2 {
			if w, err := strconv.ParseFloat(parts[1], 64); err == nil && w > 0 {
				weight = w
			}
		}
		trie.Add(word, weight)
	}
	return trie, scanner.Err()
}

// LoadTsvFile loads words and weights from a TSV file path.
func LoadTsvFile(path string) (*ThaiTrie, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	return LoadTsv(f)
}

// Add inserts a word with its weight.
func (t *ThaiTrie) Add(word string, weight float64) {
	if word == "" {
		return
	}
	t.mu.Lock()
	defer t.mu.Unlock()

	if weight > t.maxWeight {
		t.maxWeight = weight
	}

	runes := []rune(word)
	n := len(runes)

	for i := 1; i < n; i++ {
		p := string(runes[:i])
		if _, exists := t.prefixes[p]; !exists {
			t.prefixes[p] = 0.0
		}
	}

	if existing := t.prefixes[word]; weight > existing {
		t.prefixes[word] = weight
		t.totalWeight += weight - existing
	} else if _, exists := t.prefixes[word]; !exists {
		t.prefixes[word] = weight
	}
}

// TotalWeight returns the sum of the weights of all words; word probabilities
// are weight / TotalWeight().
func (t *ThaiTrie) TotalWeight() float64 {
	t.mu.RLock()
	defer t.mu.RUnlock()
	return t.totalWeight
}

// AddMany inserts multiple words with weights.
func (t *ThaiTrie) AddMany(words map[string]float64) {
	for w, weight := range words {
		t.Add(w, weight)
	}
}

// MaxWeight returns the maximum word weight stored.
func (t *ThaiTrie) MaxWeight() float64 {
	t.mu.RLock()
	defer t.mu.RUnlock()
	return t.maxWeight
}

// Prefixes finds all matching dictionary words starting at rune position `start`.
func (t *ThaiTrie) Prefixes(runes []rune, start int, maxLen int) []PrefixMatch {
	t.mu.RLock()
	defer t.mu.RUnlock()

	limit := len(runes)
	if maxLen > 0 && start+maxLen < limit {
		limit = start + maxLen
	}

	var matches []PrefixMatch
	var sb strings.Builder

	for i := start; i < limit; i++ {
		sb.WriteRune(runes[i])
		sub := sb.String()

		w, exists := t.prefixes[sub]
		if !exists {
			break
		}
		if w > 0 {
			matches = append(matches, PrefixMatch{
				End:    i + 1,
				Weight: w,
			})
		}
	}

	return matches
}

// GetWordCost calculates the negative log cost for Dijkstra / Viterbi.
func (t *ThaiTrie) GetWordCost(weight float64) float64 {
	t.mu.RLock()
	mw := t.maxWeight
	t.mu.RUnlock()

	if weight <= 0 {
		return 10.0
	}
	ratio := weight / mw
	if ratio > 1.0 {
		ratio = 1.0
	}
	if ratio < 1e-9 {
		ratio = 1e-9
	}
	return -math.Log(ratio)
}
