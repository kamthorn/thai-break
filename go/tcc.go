package thaibreak

import (
	"regexp"
	"strings"
	"sync"
	"unicode/utf8"
)

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
			"c[ั]([่-๋]c)?",
			"c[ั]([่-๋]c)?k",
			"เc็ck",
			"เcctาะk",
			"เccีtยะk",
			"เ(?:c[รลว]|หc)็ck",
			"เcิc์ck",
			"เcิtck",
			"เcีtยะ?k",
			"เcืtอะk",
			"เcืtอ?k",
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

// TCCPosArray computes valid TCC break positions for a slice of runes.
func TCCPosArray(runes []rune) []bool {
	n := len(runes)
	valid := make([]bool, n+1)
	valid[0] = true
	valid[n] = true

	if n == 0 {
		return valid
	}

	initTCCRegex()
	text := string(runes)
	byteLen := len(text)

	byteToRune := make(map[int]int, n+1)
	byteOffset := 0
	for i, r := range runes {
		byteToRune[byteOffset] = i
		byteOffset += utf8.RuneLen(r)
	}
	byteToRune[byteOffset] = n

	bytePos := 0
	for bytePos < byteLen {
		sub := text[bytePos:]

		// First, test lookahead rules if applicable
		if loc := tccLookaheadPattern.FindStringIndex(sub); loc != nil && loc[1] > 0 {
			matchLen := loc[1]
			if isFollowedByLookaheadChar(sub[matchLen:]) {
				bytePos += matchLen
				if idx, ok := byteToRune[bytePos]; ok {
					valid[idx] = true
				}
				continue
			}
		}

		// Second, test general rules
		if loc := tccGeneralPattern.FindStringIndex(sub); loc != nil && loc[1] > 0 {
			bytePos += clusterLen(sub[:loc[1]], sub[loc[1]:])
			if idx, ok := byteToRune[bytePos]; ok {
				valid[idx] = true
			}
		} else {
			_, size := utf8.DecodeRuneInString(sub)
			if size <= 0 {
				size = 1
			}
			bytePos += size
			if idx, ok := byteToRune[bytePos]; ok {
				valid[idx] = true
			}
		}
	}

	for i, r := range runes {
		if r < 0x0E00 || r > 0x0E7F {
			valid[i] = true
			valid[i+1] = true
		}
	}

	// Never a boundary before a dependent vowel or mark (e.g. inside "เมื่อ")
	for i := 1; i < n; i++ {
		if isDependentThai(runes[i]) {
			valid[i] = false
		}
	}

	return valid
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

// isDependentThai reports whether r can never start a cluster: ะ ั า ำ ิ–ฺ ๅ ็–๎.
func isDependentThai(r rune) bool {
	return (r >= 0x0E30 && r <= 0x0E3A) || r == 0x0E45 || (r >= 0x0E47 && r <= 0x0E4E)
}
