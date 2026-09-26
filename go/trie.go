package thaibreak

import (
	"bufio"
	"encoding/binary"
	"fmt"
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

// CompactDawg implements an ultra-compact Directed Acyclic Word Graph (DAWG) / Minimal DFA.
type CompactDawg struct {
	data      []byte
	offsets   []uint32
	numStates int
	numWords  int
}

// NewCompactDawg parses a binary DAWG buffer.
func NewCompactDawg(data []byte) (*CompactDawg, error) {
	if len(data) < 12 || string(data[:4]) != "TBD1" {
		return nil, fmt.Errorf("invalid DAWG header: expected TBD1 magic")
	}
	numStates := int(binary.LittleEndian.Uint32(data[4:8]))
	numWords := int(binary.LittleEndian.Uint32(data[8:12]))

	offsets := make([]uint32, numStates)
	curr := 12
	for i := 0; i < numStates; i++ {
		if curr >= len(data) {
			return nil, fmt.Errorf("corrupted DAWG data at state %d", i)
		}
		offsets[i] = uint32(curr)
		numEdges := int(data[curr] & 0x7F)
		curr += 1 + numEdges*4
	}

	return &CompactDawg{
		data:      data,
		offsets:   offsets,
		numStates: numStates,
		numWords:  numWords,
	}, nil
}

// ThaiTrie implements word lookups via either an ultra-compact DAWG or a Flat Prefix Hash Map,
// with support for runtime dynamic overlays.
type ThaiTrie struct {
	mu          sync.RWMutex
	dawg        *CompactDawg
	overlay     map[string]float64
	prefixes    map[string]float64
	maxWeight   float64
	totalWeight float64
}

// NewThaiTrie creates an empty ThaiTrie using the flat map backend.
func NewThaiTrie() *ThaiTrie {
	return &ThaiTrie{
		prefixes:  make(map[string]float64),
		maxWeight: 1.0,
	}
}

// LoadDawg loads dictionary from a pre-compiled binary DAWG byte slice.
func LoadDawg(data []byte) (*ThaiTrie, error) {
	dawg, err := NewCompactDawg(data)
	if err != nil {
		return nil, err
	}
	return &ThaiTrie{
		dawg:        dawg,
		overlay:     make(map[string]float64),
		maxWeight:   1.0,
		totalWeight: float64(dawg.numWords),
	}, nil
}

// LoadDawgFile loads dictionary from a binary DAWG file path.
func LoadDawgFile(path string) (*ThaiTrie, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	return LoadDawg(data)
}

// LoadTsv loads words and weights from a TSV reader into a flat trie.
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

// LoadFile loads dictionary, automatically choosing DAWG or TSV based on content/extension.
func LoadFile(path string) (*ThaiTrie, error) {
	if strings.HasSuffix(path, ".dawg") {
		return LoadDawgFile(path)
	}
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	var magic [4]byte
	n, _ := f.Read(magic[:])
	f.Close()
	if n == 4 && string(magic[:]) == "TBD1" {
		return LoadDawgFile(path)
	}
	return LoadTsvFile(path)
}

// Add inserts a word with its weight. Works seamlessly with both flat and DAWG backends.
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

	targetMap := t.prefixes
	if t.dawg != nil {
		if t.overlay == nil {
			t.overlay = make(map[string]float64)
		}
		targetMap = t.overlay
	}

	for i := 1; i < n; i++ {
		p := string(runes[:i])
		if _, exists := targetMap[p]; !exists {
			targetMap[p] = 0.0
		}
	}

	if existing := targetMap[word]; weight > existing {
		targetMap[word] = weight
		t.totalWeight += weight - existing
	} else if _, exists := targetMap[word]; !exists {
		targetMap[word] = weight
		t.totalWeight += weight
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

	if t.dawg != nil {
		state := 0
		d := t.dawg

		for idx := start; idx < limit; idx++ {
			r := runes[idx]
			if r > 0xFFFF {
				break
			}
			ch := uint16(r)

			off := int(d.offsets[state])
			flags := d.data[off]
			numEdges := int(flags & 0x7F)

			// Binary search edges for transition on char
			low, high := 0, numEdges-1
			nextState := -1
			base := off + 1

			for low <= high {
				mid := (low + high) / 2
				edgeOff := base + mid*4
				edgeChar := binary.LittleEndian.Uint16(d.data[edgeOff : edgeOff+2])
				if edgeChar == ch {
					nextState = int(binary.LittleEndian.Uint16(d.data[edgeOff+2 : edgeOff+4]))
					break
				} else if edgeChar < ch {
					low = mid + 1
				} else {
					high = mid - 1
				}
			}

			if nextState == -1 {
				break
			}

			state = nextState
			nextOff := int(d.offsets[state])
			if (d.data[nextOff] & 0x80) != 0 {
				matches = append(matches, PrefixMatch{
					End:    idx + 1,
					Weight: 1.0,
				})
			}
		}

		if len(t.overlay) > 0 {
			var sb strings.Builder
			for i := start; i < limit; i++ {
				sb.WriteRune(runes[i])
				sub := sb.String()
				w, exists := t.overlay[sub]
				if !exists {
					break
				}
				if w > 0 {
					found := false
					for j := range matches {
						if matches[j].End == i+1 {
							matches[j].Weight = w
							found = true
							break
						}
					}
					if !found {
						matches = append(matches, PrefixMatch{
							End:    i + 1,
							Weight: w,
						})
					}
				}
			}
		}

		return matches
	}

	// Flat Hash Trie fallback
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
