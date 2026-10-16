package thaibreak

import (
	"math/rand"
	"slices"
	"testing"
	"unicode/utf8"
)

// referenceTCCPosArray is TCCPosArray without the match window and the rules by first rune: both
// patterns run on the whole rest of the text.
func referenceTCCPosArray(runes []rune) []bool {
	n := len(runes)
	valid := make([]bool, n+1)
	valid[0] = true
	valid[n] = true
	if n == 0 {
		return valid
	}
	initTCCRegex()
	text := string(runes)
	byteToRune := make(map[int]int, n+1)
	byteOffset := 0
	for i, r := range runes {
		byteToRune[byteOffset] = i
		byteOffset += utf8.RuneLen(r)
	}
	byteToRune[byteOffset] = n
	for bytePos := 0; bytePos < len(text); {
		sub := text[bytePos:]
		if loc := tccLookaheadPattern.FindStringIndex(sub); loc != nil && loc[1] > 0 && isFollowedByLookaheadChar(sub[loc[1]:]) {
			bytePos += loc[1]
			if idx, ok := byteToRune[bytePos]; ok {
				valid[idx] = true
			}
			continue
		}
		if loc := tccGeneralPattern.FindStringIndex(sub); loc != nil && loc[1] > 0 {
			bytePos += clusterLen(sub[:loc[1]], sub[loc[1]:])
		} else {
			_, size := utf8.DecodeRuneInString(sub)
			bytePos += size
		}
		if idx, ok := byteToRune[bytePos]; ok {
			valid[idx] = true
		}
	}
	for i, r := range runes {
		if r < 0x0E00 || r > 0x0E7F {
			valid[i] = true
			valid[i+1] = true
		}
	}
	for i := 1; i < n; i++ {
		if isDependentThai(runes[i]) {
			valid[i] = false
		}
	}
	return valid
}

func TestTCCPosArrayMatchesReference(t *testing.T) {
	rng := rand.New(rand.NewSource(1))
	// consonants and vowels are more likely than the other characters, so the rules often match
	var alphabet []rune
	for r := rune(0x0E01); r <= 0x0E5B; r++ {
		alphabet = append(alphabet, r)
	}
	for r := rune(0x0E01); r <= 0x0E2E; r++ {
		alphabet = append(alphabet, r, r)
	}
	for _, r := range "เแโใไะาำิีึืุู็่้๊๋์ๆ" {
		alphabet = append(alphabet, r, r, r)
	}
	alphabet = append(alphabet, 'a', ' ', '1', '.')
	for i := 0; i < 100000; i++ {
		runes := make([]rune, 1+rng.Intn(40))
		for j := range runes {
			runes[j] = alphabet[rng.Intn(len(alphabet))]
		}
		if got, want := TCCPosArray(runes), referenceTCCPosArray(runes); !slices.Equal(got, want) {
			t.Fatalf("TCCPosArray(%q) = %v, want %v", string(runes), got, want)
		}
	}
}
