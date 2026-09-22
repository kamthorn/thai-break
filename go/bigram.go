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

// BigramModel stores bigram collocation transition counts.
type BigramModel struct {
	mu      sync.RWMutex
	bigrams map[string]float64
	alpha   float64
}

// NewBigramModel creates an empty BigramModel with default alpha = 0.15.
func NewBigramModel() *BigramModel {
	return &BigramModel{
		bigrams: make(map[string]float64),
		alpha:   0.15,
	}
}

// SetAlpha sets the scaling weight of bigram bonuses.
func (m *BigramModel) SetAlpha(alpha float64) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.alpha = alpha
}

// LoadBigramsTsv loads bigram transitions from a TSV reader (prev \t next \t count).
func LoadBigramsTsv(r io.Reader) (*BigramModel, error) {
	model := NewBigramModel()
	scanner := bufio.NewScanner(r)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		parts := strings.Split(line, "\t")
		if len(parts) < 3 {
			continue
		}
		w1, w2 := parts[0], parts[1]
		count, err := strconv.ParseFloat(parts[2], 64)
		if err == nil && count > 0 {
			model.Add(w1, w2, count)
		}
	}
	return model, scanner.Err()
}

// LoadBigramsFile loads bigrams from a TSV file path.
func LoadBigramsFile(path string) (*BigramModel, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	return LoadBigramsTsv(f)
}

// Add inserts or updates a transition count.
func (m *BigramModel) Add(w1, w2 string, count float64) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.bigrams[w1+"\t"+w2] = count
}

// GetBonus calculates transition bonus for a word pair.
// Higher frequency yields higher cost deduction.
func (m *BigramModel) GetBonus(prevWord, nextWord string, wordLen int) float64 {
	if m == nil || prevWord == "" || nextWord == "" {
		return 0.0
	}
	m.mu.RLock()
	defer m.mu.RUnlock()

	if m.alpha <= 0.0 {
		return 0.0
	}

	count, exists := m.bigrams[prevWord+"\t"+nextWord]
	if !exists {
		return 0.0
	}

	denom := float64(wordLen)
	if denom < 1.0 {
		denom = 1.0
	}
	return (m.alpha * math.Log(1.0+count)) / denom
}
