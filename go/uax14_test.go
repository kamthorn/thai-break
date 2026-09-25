package thaibreak

import "testing"

func TestLineBreakClasses(t *testing.T) {
	expected := map[rune]uint8{
		'a': lbAL, '1': lbNU, ' ': lbSP, '-': lbHY, '–': lbBA,
		'(': lbOP, ')': lbCP, '}': lbCL, '、': lbCL, '/': lbSY,
		'.': lbIS, '!': lbEX, '"': lbQU, '“': lbQI, '”': lbQF,
		'$': lbPR, '฿': lbPR, '\\': lbPR, '%': lbPO, '#': lbAL,
		'ก': lbSA, 'ๆ': lbSA, 'ั': lbSM, '่': lbSM, '๐': lbNU,
		'๚': lbBA, '中': lbID, '가': lbH2, 'א': lbHL,
		'​': lbZW, ' ': lbGL, '‍': lbZWJ, '́': lbCM,
		'😀': lbID, '\U0001F1F9': lbRI, '\U0010FFFF': lbAL,
	}
	for r, want := range expected {
		if got := lbClass(r); got != want {
			t.Errorf("class of U+%04X = %d, want %d", r, got, want)
		}
	}
	if !lbIsEastAsian('（') || lbIsEastAsian('(') {
		t.Error("East_Asian_Width flag is wrong")
	}
	if !lbIsExtPictUnassigned(0x1FFFD) || lbIsExtPictUnassigned('😀') {
		t.Error("Extended_Pictographic flag is wrong")
	}
}
