package thaibreak

import (
	"math"
	"regexp"
	"strings"
	"unicode/utf8"
)

const (
	// abbrCostFactor is the cost of an abbreviation pattern, relative to the cost of the rarest word.
	abbrCostFactor = 1.5
	// abbrLetterCostFactor is the extra cost per letter of an abbreviation
	// pattern, so "เขต|จ." beats "เข|ตจ." (a pattern taking the last letter of
	// the previous word).
	abbrLetterCostFactor = 0.01
	// unknownCostFactor is the cost of an unknown-word fallback edge, relative to the cost of the rarest word.
	unknownCostFactor = 2.0
	// tieEpsilon: costs closer than this are a tie. Edges into a position are visited from
	// the longest word to the shortest, so on a tie the later, shorter last word
	// wins and the earlier words stay longer ("ผิด|ราย" rather than "ผิ|ดราย").
	tieEpsilon = 1e-9
	maxEdges   = 50000
)

var (
	patNonThai = regexp.MustCompile(`^(?:[a-zA-Z]+(?:[-_'][a-zA-Z0-9]+)*|\d+(?:,\d+)*(?:\.\d+)?%?|[ \t]+|\r?\n|[^\x{0e00}-\x{0e7f}a-zA-Z0-9 \t\r\n])`)
	patAbbr    = regexp.MustCompile(`^(?:(?:[เแโใไ]?[ก-ฮ][ัิีึืุู็่้๊๋]?|[ก-ฮ]{1,4})\.)+`)
)

type dagEdge struct {
	from int
	word string
	cost float64
}

// Tokenizer performs weighted Thai word segmentation.
type Tokenizer struct {
	trie    *ThaiTrie
	bigrams *BigramModel
}

// NewTokenizer creates a new Tokenizer.
func NewTokenizer(trie *ThaiTrie, bigrams *BigramModel) *Tokenizer {
	return &Tokenizer{
		trie:    trie,
		bigrams: bigrams,
	}
}

func isThaiRune(r rune) bool {
	return r >= 0x0E00 && r <= 0x0E7F
}

func isThaiString(s string) bool {
	if s == "" {
		return false
	}
	for _, r := range s {
		if !isThaiRune(r) {
			return false
		}
	}
	return true
}

// Tokenize segments text into word tokens.
func (tok *Tokenizer) Tokenize(text string, keepWhitespace bool) []string {
	if text == "" {
		return nil
	}

	runes := []rune(text)
	n := len(runes)
	if n == 0 {
		return nil
	}

	validPos := TCCPosArray(runes)

	// Precompute byte offsets for regex matching on string slices
	charByteOffsets := make([]int, n+1)
	b := 0
	for i, r := range runes {
		charByteOffsets[i] = b
		b += utf8.RuneLen(r)
	}
	charByteOffsets[n] = b

	// Unigram costs: -log(weight / total); a word of weight 1 costs rareCost.
	// Non-Thai tokens cost rareCost, abbreviation patterns and unknown-word
	// fallbacks more, so a dictionary word followed by "." beats a pattern such
	// as "ว." that would cut the word.
	normalizer := tok.trie.TotalWeight() + 1.0
	rareCost := math.Log(normalizer)

	// Pass 1: Collect edgesTo[j]
	edgesTo := make([][]dagEdge, n+1)
	edgeCount := 0

CollectLoop:
	for i := 0; i < n; i++ {
		if !validPos[i] {
			continue
		}

		bytePos := charByteOffsets[i]
		subText := text[bytePos:]

		if isThaiRune(runes[i]) {
			// 1. Thai dictionary words starting at i
			matches := tok.trie.Prefixes(runes, i, 25)
			for _, m := range matches {
				j := m.End
				if j > n || !validPos[j] {
					continue
				}
				word := string(runes[i:j])
				edgesTo[j] = append(edgesTo[j], dagEdge{from: i, word: word, cost: math.Log(normalizer / m.Weight)})
				edgeCount++
				if edgeCount >= maxEdges {
					break CollectLoop
				}
			}

			// 2. Thai abbreviation patterns
			if loc := patAbbr.FindStringIndex(subText); loc != nil && loc[0] == 0 {
				mStr := subText[:loc[1]]
				abbrLen := len([]rune(mStr))
				j := i + abbrLen
				if j <= n && validPos[j] {
					letters := abbrLen - strings.Count(mStr, ".")
					cost := (abbrCostFactor + abbrLetterCostFactor*float64(letters)) * rareCost
					edgesTo[j] = append(edgesTo[j], dagEdge{from: i, word: mStr, cost: cost})
				}
			}
		} else {
			// 3. Non-Thai tokens
			if loc := patNonThai.FindStringIndex(subText); loc != nil && loc[0] == 0 {
				mStr := subText[:loc[1]]
				wordLen := len([]rune(mStr))
				j := i + wordLen
				if j <= n {
					edgesTo[j] = append(edgesTo[j], dagEdge{from: i, word: mStr, cost: rareCost})
				}
			}
		}
	}

	// Pass 2: Viterbi forward DP
	dp := make([]float64, n+1)
	from := make([]int, n+1)
	word := make([]string, n+1)
	isUnk := make([]bool, n+1)

	for i := range dp {
		dp[i] = math.Inf(1)
		from[i] = -1
	}
	dp[0] = 0.0

	for j := 1; j <= n; j++ {
		if !validPos[j] {
			continue
		}

		// Try all dictionary/pattern edges ending at j
		if len(edgesTo[j]) > 0 {
			for _, e := range edgesTo[j] {
				i := e.from
				if math.IsInf(dp[i], 1) {
					continue
				}
				wLen := j - i
				if wLen < 1 {
					wLen = 1
				}
				baseCost := e.cost
				edgeCost := baseCost
				if tok.bigrams != nil && word[i] != "" {
					bonus := tok.bigrams.GetBonus(word[i], e.word, wLen)
					edgeCost = math.Max(0.01, baseCost-bonus)
				}

				newCost := dp[i] + edgeCost
				if newCost <= dp[j]+tieEpsilon {
					dp[j] = newCost
					from[j] = i
					word[j] = e.word
					isUnk[j] = false
				}
			}
		}

		// Unknown-word fallback: connect to nearest reachable predecessor
		if math.IsInf(dp[j], 1) {
			for i := j - 1; i >= 0; i-- {
				if !math.IsInf(dp[i], 1) && validPos[i] {
					unknownWord := string(runes[i:j])
					newCost := dp[i] + unknownCostFactor*rareCost
					dp[j] = newCost
					from[j] = i
					word[j] = unknownWord
					isUnk[j] = true
					break
				}
			}
		}
	}

	// Traceback
	if math.IsInf(dp[n], 1) {
		return []string{text}
	}

	var rawTokens []string
	var rawIsUnk []bool
	pos := n
	for pos > 0 {
		rawTokens = append(rawTokens, word[pos])
		rawIsUnk = append(rawIsUnk, isUnk[pos])
		pos = from[pos]
		if pos < 0 {
			break
		}
	}

	// Reverse
	for l, r := 0, len(rawTokens)-1; l < r; l, r = l+1, r-1 {
		rawTokens[l], rawTokens[r] = rawTokens[r], rawTokens[l]
		rawIsUnk[l], rawIsUnk[r] = rawIsUnk[r], rawIsUnk[l]
	}

	// Syllable-based OOV chunking: merge consecutive unknown Thai clusters
	var tokens []string
	var curChunk strings.Builder

	for idx, tokStr := range rawTokens {
		if rawIsUnk[idx] && isThaiString(tokStr) {
			curChunk.WriteString(tokStr)
		} else {
			if curChunk.Len() > 0 {
				tokens = append(tokens, curChunk.String())
				curChunk.Reset()
			}
			tokens = append(tokens, tokStr)
		}
	}
	if curChunk.Len() > 0 {
		tokens = append(tokens, curChunk.String())
	}

	if !keepWhitespace {
		var filtered []string
		for _, t := range tokens {
			if strings.TrimSpace(t) != "" {
				filtered = append(filtered, t)
			}
		}
		return filtered
	}

	return tokens
}
