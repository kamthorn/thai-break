package thaibreak

import (
	"regexp"
	"regexp/syntax"
	"strings"
	"sync"
	"unicode/utf8"
)

var (
	tccGeneralPattern *regexp.Regexp
	// tccGeneralByClass has, for each class of first rune (see tccFirstClass), the general rules that
	// can start with it, in the order of tccGeneralPattern, so an alternative that cannot match is
	// never tried. The result is the same as that of tccGeneralPattern.
	tccGeneralByClass   [tccClasses]*regexp.Regexp
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
		for class := 0; class < tccClasses; class++ {
			var sel []string
			for _, p := range genPatterns {
				if tccCanStartWith(p, class) {
					sel = append(sel, p)
				}
			}
			tccGeneralByClass[class] = regexp.MustCompile("^(" + strings.Join(sel, "|") + ")")
		}
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

// tccClasses is the number of classes of first rune of a TCC cluster: a consonant, เ, แ, โ, and ใ or ไ.
const tccClasses = 5

// tccFirstClass returns the class of r (see tccClasses), or -1 if no general rule starts with r.
func tccFirstClass(r rune) int {
	switch {
	case r >= 0x0E01 && r <= 0x0E2E:
		return 0
	case r >= 0x0E40 && r <= 0x0E42:
		return int(r-0x0E40) + 1
	case r == 0x0E43 || r == 0x0E44:
		return 4
	}
	return -1
}

// tccClassRange is the range of runes of a class of first rune.
var tccClassRange = [tccClasses][2]rune{{0x0E01, 0x0E2E}, {0x0E40, 0x0E40}, {0x0E41, 0x0E41}, {0x0E42, 0x0E42}, {0x0E43, 0x0E44}}

// tccCanStartWith reports whether the rule pattern can match a string that starts with a rune of the
// class. It errs on the side of true.
func tccCanStartWith(pattern string, class int) bool {
	re, err := syntax.Parse(pattern, syntax.Perl)
	if err != nil {
		return true
	}
	ranges, ok := tccFirstRunes(re)
	if !ok {
		return true
	}
	lo, hi := tccClassRange[class][0], tccClassRange[class][1]
	for _, r := range ranges {
		if r[0] <= hi && lo <= r[1] {
			return true
		}
	}
	return false
}

// tccFirstRunes returns the ranges of runes a match of re can start with, or false if unknown.
func tccFirstRunes(re *syntax.Regexp) ([][2]rune, bool) {
	switch re.Op {
	case syntax.OpLiteral:
		if re.Flags&syntax.FoldCase != 0 || len(re.Rune) == 0 {
			return nil, false
		}
		return [][2]rune{{re.Rune[0], re.Rune[0]}}, true
	case syntax.OpCharClass:
		var out [][2]rune
		for i := 0; i+1 < len(re.Rune); i += 2 {
			out = append(out, [2]rune{re.Rune[i], re.Rune[i+1]})
		}
		return out, true
	case syntax.OpConcat:
		return tccFirstRunes(re.Sub[0])
	case syntax.OpCapture:
		return tccFirstRunes(re.Sub[0])
	case syntax.OpAlternate:
		var out [][2]rune
		for _, sub := range re.Sub {
			r, ok := tccFirstRunes(sub)
			if !ok {
				return nil, false
			}
			out = append(out, r...)
		}
		return out, true
	}
	return nil, false
}

// tccMaxRunes bounds the length of a match of the TCC patterns (the longest has 11 runes).
const tccMaxRunes = 16

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

	// Rune index at each rune start (by byte offset), and the byte offset of each rune
	byteToRune := make([]int32, byteLen+1)
	runeOffset := make([]int, n+1)
	byteOffset := 0
	for i, r := range runes {
		byteToRune[byteOffset] = int32(i)
		runeOffset[i] = byteOffset
		byteOffset += utf8.RuneLen(r)
	}
	byteToRune[byteOffset] = int32(n)
	runeOffset[n] = byteOffset

	bytePos := 0
	for bytePos < byteLen {
		sub := text[bytePos:]
		// No rule matches more than tccMaxRunes runes, so the patterns only see this window: the
		// regexp package is much slower on a long input
		window := text[bytePos:runeOffset[min(int(byteToRune[bytePos])+tccMaxRunes, n)]]

		// First, test lookahead rules if applicable (all of them start with เ)
		if sub[0] == 0xE0 && strings.HasPrefix(sub, "เ") {
			if loc := tccLookaheadPattern.FindStringIndex(window); loc != nil && loc[1] > 0 {
				matchLen := loc[1]
				if isFollowedByLookaheadChar(sub[matchLen:]) {
					bytePos += matchLen
					valid[byteToRune[bytePos]] = true
					continue
				}
			}
		}

		// Second, test general rules
		var general *regexp.Regexp
		if class := tccFirstClass(runes[byteToRune[bytePos]]); class >= 0 {
			general = tccGeneralByClass[class]
		}
		var loc []int
		if general != nil {
			loc = general.FindStringIndex(window)
		}
		if loc != nil && loc[1] > 0 {
			bytePos += clusterLen(sub[:loc[1]], sub[loc[1]:])
			valid[byteToRune[bytePos]] = true
		} else {
			_, size := utf8.DecodeRuneInString(sub)
			if size <= 0 {
				size = 1
			}
			bytePos += size
			valid[byteToRune[bytePos]] = true
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
