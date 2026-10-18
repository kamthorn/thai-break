package thaibreak

// tccDFA is an anchored leftmost-first DFA over classes of the Thai block (U+0E00..U+0E7F). The
// tables in tcc_dfa.go are generated from the cluster rules of the Rust core (rust/src/tcc.rs) by
// thaibreak_gen_tcc. A match is the match of the first rule that matches, as with a regexp
// alternation of the rules (see the reference in tcc_test.go).
type tccDFA struct {
	classes int
	start   uint16
	class   [128]uint8
	trans   []uint16
	match   []bool
}

// find returns the end of the match that starts at runes[start], or -1.
func (d *tccDFA) find(runes []rune, start int) int {
	state, found := d.start, -1
	for p := start; p < len(runes); p++ {
		off := uint32(runes[p]) - 0x0E00
		if off >= 128 {
			break
		}
		state = d.trans[int(state)*d.classes+int(d.class[off])]
		if state == 0 {
			break
		}
		if d.match[state] {
			found = p + 1
		}
	}
	return found
}

// canFollowLookaheadCluster reports whether a lookahead cluster (-ีย) may end before runes[i]:
// at the end of the text, or before a consonant (ก-ฮ), a leading vowel (เ-ไ) or a non-Thai rune.
func canFollowLookaheadCluster(runes []rune, i int) bool {
	if i == len(runes) {
		return true
	}
	r := runes[i]
	return (r >= 0x0E01 && r <= 0x0E2E) || (r >= 0x0E40 && r <= 0x0E44) || r < 0x0E00 || r > 0x0E7F
}

// TCCPosArray computes valid TCC break positions for a slice of runes.
func TCCPosArray(runes []rune) []bool {
	n := len(runes)
	valid := make([]bool, n+1)
	valid[0] = true
	valid[n] = true

	for pos := 0; pos < n; {
		// First, test lookahead rules (all of them start with เ)
		if runes[pos] == 'เ' {
			if end := tccLookaheadDFA.find(runes, pos); end >= 0 && canFollowLookaheadCluster(runes, end) {
				pos = end
				valid[pos] = true
				continue
			}
		}

		// Second, test general rules; otherwise advance one rune
		if end := tccGeneralDFA.find(runes, pos); end >= 0 {
			pos = clusterEnd(runes, pos, end)
		} else {
			pos++
		}
		valid[pos] = true
	}

	for i, r := range runes {
		if r < 0x0E00 || r > 0x0E7F {
			valid[i] = true
			valid[i+1] = true
		} else if i > 0 && isDependentThai(r) {
			// Never a boundary before a dependent vowel or mark (e.g. inside "เมื่อ")
			valid[i] = false
		}
	}

	return valid
}

// clusterEnd returns the end of a matched cluster runes[start:end]. A final consonant followed by a
// dependent vowel or mark starts the next cluster instead: "รึยัง" is "รึ" + "ยัง", not "รึย" + "ัง".
// Except ว before ะ, which is part of the vowel -ัวะ ("ผัวะ").
func clusterEnd(runes []rune, start, end int) int {
	if end < len(runes) && end-start > 1 {
		last, next := runes[end-1], runes[end]
		if last >= 'ก' && last <= 'ฮ' && isDependentThai(next) && !(last == 'ว' && next == 'ะ') {
			return end - 1
		}
	}
	return end
}

// isDependentThai reports whether r can never start a cluster: ะ ั า ำ ิ–ฺ ๅ ็–๎.
func isDependentThai(r rune) bool {
	return (r >= 0x0E30 && r <= 0x0E3A) || r == 0x0E45 || (r >= 0x0E47 && r <= 0x0E4E)
}
