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
