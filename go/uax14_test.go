package thaibreak

import (
	"bufio"
	"os"
	"strconv"
	"strings"
	"testing"
)

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

// TestLineBreakTestConformance runs the official conformance test (SA resolved
// to AL, no dictionary).
func TestLineBreakTestConformance(t *testing.T) {
	f, err := os.Open("../testdata/LineBreakTest-16.0.0.txt")
	if err != nil {
		t.Skip("LineBreakTest data not found")
	}
	defer f.Close()

	failures := 0
	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		line := scanner.Text()
		if line == "" || line[0] == '#' {
			continue
		}
		var cps []rune
		var want []bool
		for _, field := range strings.Fields(line) {
			switch field {
			case "×":
				want = append(want, false)
			case "÷":
				want = append(want, true)
			default:
				cp, _ := strconv.ParseUint(field, 16, 32)
				cps = append(cps, rune(cp))
			}
		}
		got := lbBreakOpportunities(cps, nil)
		for i := range want {
			if (got[i] != lbNoBreak) != want[i] {
				if failures++; failures <= 10 {
					t.Errorf("mismatch at position %d: %s", i, line)
				}
				break
			}
		}
	}
	if err := scanner.Err(); err != nil {
		t.Fatal(err)
	}
}
