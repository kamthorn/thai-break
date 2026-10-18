package thaibreak

import (
	"math/rand"
	"regexp"
	"slices"
	"strings"
	"sync"
	"testing"
	"unicode/utf8"
)

// The regexp implementation that the generated DFA replaced, kept as the reference: the rules
// below are the Go port's own, so the test also catches tables that are out of date.
var (
	tccGeneralPattern   *regexp.Regexp
	tccLookaheadPattern *regexp.Regexp
	tccOnce             sync.Once
)

func initTCCRegex() {
	tccOnce.Do(func() {
		c := "[ก-ฮ]"
		t := "[่-๋]?"
		d := "[ุู]"
		k := "([ก-ฮ][ก-ฮ]?[ุูิ]?์)?"

		rawGeneralRules := []string{
			// Mai Han-akat always has a final: -ัวะ, or a consonant (with or without a tone mark).
			"cั[่-๋]?วะ",
			"cั[่-๋]?ck",
			"cั[่-๋]?",
			"เc็ck",
			"เcctาะk",
			"เccีtยะk",
			"เ(?:c[รลว]|หc)็ck",
			"เcิc์ck",
			// A true cluster or ห-led onset between เ and the vowel belongs to the syllable (เครื่อง, เพลิง, เปล่า).
			"เ(?:[กขคตปพผบด][รล]|[กขค]ว|ทร|ห[งญนมยรลว])ิtck",
			"เcิtck",
			"เcีtยะ?k",
			"เ(?:[กขคตปพผบด][รล]|[กขค]ว|ทร|ห[งญนมยรลว])ืtอะ?k",
			"เcืtอะk",
			"เcืtอ?k",
			"เ(?:[กขคตปพผบด][รล]|[กขค]ว|ทร|ห[งญนมยรลว])tาk",
			"เctา?ะ?k",
			"c[ึื]tck",
			"c[ะ-ู]tk",
			"c[ิุู]์",
			"cรรc์",
			"c็",
			"ct[ะาำ]?k",
			"แc็ck",
			"แcc์k",
			"แctะk",
			"แ(?:c[รลว]|หc)็ck",
			"แccc์k",
			"โctะk",
			"[เ-ไ]ctk",
			"ก็",
			"อึ",
			"หึ",
		}

		rawLookaheadRules := []string{
			"เccีtยk",
			"เc[ิีุู]tยk",
		}

		expand := func(rules []string) []string {
			res := make([]string, len(rules))
			for i, rule := range rules {
				p := strings.ReplaceAll(rule, "k", k)
				p = strings.ReplaceAll(p, "c", c)
				p = strings.ReplaceAll(p, "t", t)
				p = strings.ReplaceAll(p, "d", d)
				res[i] = p
			}
			return res
		}

		genPatterns := expand(rawGeneralRules)
		lookPatterns := expand(rawLookaheadRules)

		tccGeneralPattern = regexp.MustCompile("^(" + strings.Join(genPatterns, "|") + ")")
		tccLookaheadPattern = regexp.MustCompile("^(" + strings.Join(lookPatterns, "|") + ")")
	})
}

// isFollowedByLookaheadChar checks if next rune can follow diphthong -ia.
// Specifically: end of string, base consonants (ก-ฮ), leading vowels (เ-ไ),
// whitespace, or punctuation.
func isFollowedByLookaheadChar(rest string) bool {
	if len(rest) == 0 {
		return true
	}
	r, _ := utf8.DecodeRuneInString(rest)
	if (r >= 0x0E01 && r <= 0x0E2E) || (r >= 0x0E40 && r <= 0x0E44) {
		return true
	}
	if r < 0x0E00 || r > 0x0E7F {
		return true
	}
	return false
}

// clusterLen returns the byte length of a matched cluster. A final consonant
// followed by a dependent vowel or mark starts the next cluster instead:
// "รึยัง" is "รึ" + "ยัง", not "รึย" + "ัง". Except ว before ะ, which is part
// of the vowel -ัวะ ("ผัวะ").
func clusterLen(match, rest string) int {
	last, size := utf8.DecodeLastRuneInString(match)
	next, _ := utf8.DecodeRuneInString(rest)
	if utf8.RuneCountInString(match) > 1 && last >= 'ก' && last <= 'ฮ' && isDependentThai(next) && !(last == 'ว' && next == 'ะ') {
		return len(match) - size
	}
	return len(match)
}

// referenceTCCPosArray computes TCCPosArray with the regexp rules.
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
