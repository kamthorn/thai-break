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
)

var (
	patNonThai = regexp.MustCompile(`^(?:[a-zA-Z]+(?:[-_'][a-zA-Z0-9]+)*|\d+(?:,\d+)*(?:\.\d+)?%?|[ \t]+|\r?\n|[^\x{0e00}-\x{0e7f}a-zA-Z0-9 \t\r\n])`)
	patAbbr    = regexp.MustCompile(`^(?:(?:[เแโใไ]?[ก-ฮ][ัิีึืุู็่้๊๋]?|[ก-ฮ]{1,4})\.)+`)
)

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

	// Match against a normalized copy, but return the original characters
	runes := []rune(text)
	norm, origPos := normalizeForMatching(runes)
	tokens := tok.segment(norm)
	if len(norm) != len(runes) || string(norm) != text {
		original := make([]string, 0, len(tokens))
		pos := 0
		for _, t := range tokens {
			end := pos + utf8.RuneCountInString(t)
			original = append(original, string(runes[origPos[pos]:origPos[end]]))
			pos = end
		}
		tokens = original
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

// normalizeForMatching normalizes common Thai spelling variants for
// dictionary matching: เ + เ → แ, ํ + า → ำ, ํ + tone + า → tone + ำ
// (e.g. "นํ้า" → "น้ำ"). It returns the normalized runes and, for each of
// them, the index of the original rune it starts at (plus a final entry for
// the end). Token boundaries never fall inside a replaced pair, so tokens map
// back to exact slices of the original text.
func normalizeForMatching(runes []rune) ([]rune, []int) {
	n := len(runes)
	norm := make([]rune, 0, n)
	orig := make([]int, 0, n+1)
	at := func(i int) rune {
		if i < n {
			return runes[i]
		}
		return 0
	}
	for i := 0; i < n; i++ {
		switch next := at(i + 1); {
		case runes[i] == 'เ' && next == 'เ':
			norm, orig = append(norm, 'แ'), append(orig, i)
			i++
		case runes[i] == '\u0E4D' && next == 'า':
			norm, orig = append(norm, 'ำ'), append(orig, i)
			i++
		case runes[i] == '\u0E4D' && next >= '่' && next <= '๋' && at(i+2) == 'า':
			norm, orig = append(norm, next, 'ำ'), append(orig, i, i)
			i += 2
		default:
			norm, orig = append(norm, runes[i]), append(orig, i)
		}
	}
	return norm, append(orig, n)
}

// segment runs the Viterbi segmentation over runes and returns all tokens,
// including whitespace.
func (tok *Tokenizer) segment(runes []rune) []string {
	text := string(runes)
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

	// Single-pass Viterbi DP. Positions are visited in order. When position i
	// is reached, every edge into it has been relaxed, so dp[i] is final: an
	// unreachable i gets an unknown-word edge, then the edges starting at i are
	// relaxed right away. Edges are never stored, so memory stays O(n) for any
	// text length.
	dp := make([]float64, n+1)
	from := make([]int, n+1)
	word := make([]string, n+1)
	isUnk := make([]bool, n+1)
	for i := range dp {
		dp[i] = math.Inf(1)
		from[i] = -1
	}
	dp[0] = 0.0

	type outEdge struct {
		to   int
		word string
		cost float64
	}
	var edges []outEdge

	for i := 0; i <= n; i++ {
		if !validPos[i] {
			continue
		}

		// Unknown-word fallback: connect to nearest reachable predecessor
		if math.IsInf(dp[i], 1) {
			for k := i - 1; k >= 0; k-- {
				if !math.IsInf(dp[k], 1) && validPos[k] {
					dp[i] = dp[k] + unknownCostFactor*rareCost
					from[i] = k
					word[i] = string(runes[k:i])
					isUnk[i] = true
					break
				}
			}
		}

		if i == n {
			break
		}

		// Edges starting at i
		edges = edges[:0]
		subText := text[charByteOffsets[i]:]

		if isThaiRune(runes[i]) {
			// 1. Thai dictionary words starting at i
			for _, m := range tok.trie.Prefixes(runes, i, 25) {
				if j := m.End; j <= n && validPos[j] {
					edges = append(edges, outEdge{j, string(runes[i:j]), math.Log(normalizer / m.Weight)})
				}
			}

			// 2. Thai abbreviation patterns
			if loc := patAbbr.FindStringIndex(subText); loc != nil && loc[0] == 0 {
				mStr := subText[:loc[1]]
				abbrLen := utf8.RuneCountInString(mStr)
				if j := i + abbrLen; j <= n && validPos[j] {
					letters := abbrLen - strings.Count(mStr, ".")
					edges = append(edges, outEdge{j, mStr, (abbrCostFactor + abbrLetterCostFactor*float64(letters)) * rareCost})
				}
			}
		} else {
			// 3. Non-Thai tokens
			if loc := patNonThai.FindStringIndex(subText); loc != nil && loc[0] == 0 {
				mStr := subText[:loc[1]]
				if j := i + utf8.RuneCountInString(mStr); j <= n && validPos[j] {
					edges = append(edges, outEdge{j, mStr, rareCost})
				}
			}
		}

		// Relax the edges
		for _, e := range edges {
			edgeCost := e.cost
			if tok.bigrams != nil && word[i] != "" {
				bonus := tok.bigrams.GetBonus(word[i], e.word, e.to-i)
				edgeCost = math.Max(0.01, e.cost-bonus)
			}

			// Ties go to the later start, i.e. the shorter last word (see tieEpsilon)
			if newCost := dp[i] + edgeCost; newCost <= dp[e.to]+tieEpsilon {
				dp[e.to] = newCost
				from[e.to] = i
				word[e.to] = e.word
				isUnk[e.to] = false
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

	return tokens
}
