package thaibreak

// Unicode Line Breaking Algorithm (UAX #14) for Unicode 16.0.
//
// Classes are resolved as in LB1 (AI, SG, XX → AL; CJ → NS), except that
// Complex_Context (SA) is kept as lbSA (letters) and lbSM (combining marks) so
// Thai runs can be segmented with the dictionary. QU is split into lbQI
// (\p{Pi}), lbQF (\p{Pf}) and plain lbQU for LB15a, LB15b and LB19.

const (
	lbAL uint8 = iota
	lbBK
	lbCR
	lbLF
	lbNL
	lbSP
	lbZW
	lbWJ
	lbGL
	lbCL
	lbCP
	lbEX
	lbIS
	lbSY
	lbOP
	lbQU
	lbQI
	lbQF
	lbNS
	lbB2
	lbBA
	lbBB
	lbHY
	lbHH
	lbCB
	lbIN
	lbHL
	lbNU
	lbPR
	lbPO
	lbID
	lbEB
	lbEM
	lbH2
	lbH3
	lbJL
	lbJV
	lbJT
	lbRI
	lbZWJ
	lbCM
	lbAK
	lbAP
	lbAS
	lbVF
	lbVI
	lbSA
	lbSM
)

const (
	lbClassMask             = 0x3F
	lbFlagEastAsian         = 0x40
	lbFlagExtPictUnassigned = 0x80
)

// lbValue returns the packed table value (class id and flags) for a code point.
func lbValue(r rune) uint8 {
	cp := uint32(r)
	lo, hi := 0, len(lbStarts)-1
	for lo < hi {
		mid := (lo + hi + 1) >> 1
		if lbStarts[mid] <= cp {
			lo = mid
		} else {
			hi = mid - 1
		}
	}
	return lbValues[lo]
}

// lbClass returns the resolved Line_Break class of a code point.
func lbClass(r rune) uint8 { return lbValue(r) & lbClassMask }

// lbIsEastAsian reports whether East_Asian_Width is F, W or H (LB19a, LB30).
func lbIsEastAsian(r rune) bool { return lbValue(r)&lbFlagEastAsian != 0 }

// lbIsExtPictUnassigned reports whether r is Extended_Pictographic and unassigned (LB30b).
func lbIsExtPictUnassigned(r rune) bool { return lbValue(r)&lbFlagExtPictUnassigned != 0 }

// Break actions returned by lbBreakOpportunities.
const (
	lbNoBreak uint8 = iota
	lbAllowed
	lbMandatory
)

// lbNone is the class of a position outside the text (sot / eot).
const lbNone uint8 = 0xFF

const (
	thaiMaiyamok  = 0x0E46
	thaiPaiyannoi = 0x0E2F
	hyphen        = 0x2010
	dottedCircle  = 0x25CC
)

// lbUnit is a character with its attached combining marks (LB9).
type lbUnit struct {
	cls   uint8
	cp    rune
	start int
	ea    bool
	xp    bool
	zwj   bool
}

// lbBreakOpportunities computes the break action before every position of cps.
//
// Between two SA letters (Thai, Lao, Khmer, Myanmar) the dictionary decides:
// a break is allowed where dictBreaks[i] is true. As a Thai tailoring, there
// is never a break before ๆ (U+0E46) or ฯ (U+0E2F). When dictBreaks is nil,
// SA is resolved to AL as in LB1 and no tailoring is applied.
//
// The result has len(cps)+1 entries: position 0 is never a break (LB2) and
// position len(cps) always is (LB3).
func lbBreakOpportunities(cps []rune, dictBreaks []bool) []uint8 {
	n := len(cps)
	actions := make([]uint8, n+1)
	if n == 0 {
		return actions
	}
	actions[n] = lbMandatory

	// LB9/LB10: group X (CM|ZWJ)* into units that take the class of X.
	units := make([]lbUnit, 0, n)
	for i, cp := range cps {
		v := lbValue(cp)
		cls := v & lbClassMask
		if cls == lbSM {
			cls = lbCM
		}
		if cls == lbCM || cls == lbZWJ {
			if last := len(units) - 1; last >= 0 && !lbIsOneOf(units[last].cls, lbBK, lbCR, lbLF, lbNL, lbSP, lbZW) {
				units[last].zwj = cls == lbZWJ
				continue
			}
		}
		resolved := cls
		switch {
		case cls == lbCM || cls == lbZWJ:
			resolved = lbAL
		case cls == lbSA && dictBreaks == nil:
			resolved = lbAL
		}
		units = append(units, lbUnit{
			cls:   resolved,
			cp:    cp,
			start: i,
			ea:    v&lbFlagEastAsian != 0,
			xp:    v&lbFlagExtPictUnassigned != 0,
			zwj:   cls == lbZWJ,
		})
	}

	for u := 1; u < len(units); u++ {
		actions[units[u].start] = lbPairAction(units, u, dictBreaks)
	}
	return actions
}

// lbPairAction applies rules LB4–LB31 in order to the boundary before unit b.
func lbPairAction(units []lbUnit, b int, dictBreaks []bool) uint8 {
	a := b - 1
	count := len(units)
	cls := func(i int) uint8 {
		if i < 0 || i >= count {
			return lbNone
		}
		if units[i].cls == lbSA {
			return lbAL
		}
		return units[i].cls
	}
	A, B := units[a].cls, units[b].cls

	// LB4–LB6: mandatory breaks
	if A == lbBK {
		return lbMandatory
	}
	if A == lbCR && B == lbLF {
		return lbNoBreak
	}
	if A == lbCR || A == lbLF || A == lbNL {
		return lbMandatory
	}
	if lbIsOneOf(B, lbBK, lbCR, lbLF, lbNL) {
		return lbNoBreak
	}
	// LB7: × SP, × ZW
	if B == lbSP || B == lbZW {
		return lbNoBreak
	}
	// Last unit before the boundary, skipping spaces (for the "X SP*" rules)
	k := a
	for k > 0 && units[k].cls == lbSP {
		k--
	}
	K := cls(k)
	// LB8: ZW SP* ÷
	if K == lbZW {
		return lbAllowed
	}
	// LB8a: ZWJ ×
	if units[a].zwj {
		return lbNoBreak
	}
	// SA resolution (LB1), Thai tailoring: ๆ and ฯ never start a line, even after spaces
	if dictBreaks != nil && (units[b].cp == thaiMaiyamok || units[b].cp == thaiPaiyannoi) {
		return lbNoBreak
	}
	// SA resolution (LB1): the dictionary decides inside SA runs
	if A == lbSA && B == lbSA {
		if dictBreaks[units[b].start] {
			return lbAllowed
		}
		return lbNoBreak
	}
	A, B = cls(a), cls(b)
	// LB13: × CL, × CP, × EX, × SY
	if lbIsOneOf(B, lbCL, lbCP, lbEX, lbSY) {
		return lbNoBreak
	}
	// LB15c: SP ÷ IS NU
	if A == lbSP && B == lbIS && cls(b+1) == lbNU {
		return lbAllowed
	}
	// LB15d: × IS
	if B == lbIS {
		return lbNoBreak
	}
	// LB18: SP ÷
	if A == lbSP {
		return lbAllowed
	}
	// LB20a: (sot | BK | CR | LF | NL | SP | ZW | CB | GL) (HY | [‐]) × AL
	if (A == lbHY || units[a].cp == hyphen) && B == lbAL &&
		(a == 0 || lbIsOneOf(cls(a-1), lbBK, lbCR, lbLF, lbNL, lbSP, lbZW, lbCB, lbGL)) {
		return lbNoBreak
	}
	// LB21: × BA, × HY, × NS, BB ×
	if lbIsOneOf(B, lbBA, lbHY, lbNS) || A == lbBB {
		return lbNoBreak
	}
	// LB21a: HL (HY | [BA - $EastAsian]) × [^HL]
	if (A == lbHY || (A == lbBA && !units[a].ea)) && cls(a-1) == lbHL && B != lbHL {
		return lbNoBreak
	}
	// LB21b: SY × HL
	if A == lbSY && B == lbHL {
		return lbNoBreak
	}
	// LB24: (PR | PO) × (AL | HL), (AL | HL) × (PR | PO)
	if ((A == lbPR || A == lbPO) && lbIsAlpha(B)) || (lbIsAlpha(A) && (B == lbPR || B == lbPO)) {
		return lbNoBreak
	}
	// LB25: numbers
	if B == lbPO || B == lbPR {
		// NU (SY | IS)* (CL | CP)? × (PO | PR)
		j := a
		if A == lbCL || A == lbCP {
			j--
		}
		for cls(j) == lbSY || cls(j) == lbIS {
			j--
		}
		if cls(j) == lbNU {
			return lbNoBreak
		}
	}
	if (A == lbPO || A == lbPR) && B == lbOP && (cls(b+1) == lbNU || (cls(b+1) == lbIS && cls(b+2) == lbNU)) {
		return lbNoBreak // (PO | PR) × OP IS? NU
	}
	if B == lbNU && lbIsOneOf(A, lbPO, lbPR, lbHY, lbIS) {
		return lbNoBreak // (PO | PR | HY | IS) × NU
	}
	if B == lbNU {
		// NU (SY | IS)* × NU
		j := a
		for cls(j) == lbSY || cls(j) == lbIS {
			j--
		}
		if cls(j) == lbNU {
			return lbNoBreak
		}
	}
	// LB27: (JL | JV | JT | H2 | H3) × PO, PR × (JL | JV | JT | H2 | H3)
	if (lbIsHangul(A) && B == lbPO) || (A == lbPR && lbIsHangul(B)) {
		return lbNoBreak
	}
	// LB31: ÷
	return lbAllowed
}

// lbInAksara implements LB28a for the boundary between units a and b.
func lbInAksara(units []lbUnit, a, b int, cls func(int) uint8) bool {
	aksara := func(i int) bool {
		return cls(i) == lbAK || cls(i) == lbAS || (i >= 0 && i < len(units) && units[i].cp == dottedCircle)
	}
	A, B := cls(a), cls(b)
	return (A == lbAP && aksara(b)) ||
		(aksara(a) && (B == lbVF || B == lbVI)) ||
		(A == lbVI && aksara(a-1) && (B == lbAK || units[b].cp == dottedCircle)) ||
		(aksara(a) && aksara(b) && cls(b+1) == lbVF)
}

func lbIsOneOf(c uint8, set ...uint8) bool {
	for _, s := range set {
		if c == s {
			return true
		}
	}
	return false
}

func lbIsAlpha(c uint8) bool { return c == lbAL || c == lbHL }

func lbIsQuote(c uint8) bool { return c == lbQU || c == lbQI || c == lbQF }

func lbIsHangul(c uint8) bool { return lbIsOneOf(c, lbJL, lbJV, lbJT, lbH2, lbH3) }
