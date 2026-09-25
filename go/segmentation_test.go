package thaibreak

import (
	"strings"
	"testing"
)

// segmentationCases maps input to the expected words joined by '|'.
var segmentationCases = []struct{ name, input, want string }{
	{"a word before a full stop is not cut into an abbreviation", "เขากินข้าว.", "เขา|กิน|ข้าว|."},
	{"a word before a full stop, then more text", "ฉันรักเธอ.ไปเที่ยวกัน", "ฉัน|รัก|เธอ|.|ไป|เที่ยว|กัน"},
	{"abbreviations are still recognized", "เมื่อ 5 มิ.ย. ที่ จ.พิษณุโลก", "เมื่อ|5|มิ.ย.|ที่|จ.|พิษณุโลก"},
	{"an abbreviation after a word keeps the word whole", "ในเขตจ.พิจิตร", "ใน|เขต|จ.|พิจิตร"},
	{"an abbreviation after a word that ends like one", "ในเดือนพ.ย.", "ใน|เดือน|พ.ย."},
}

func TestSegmentation(t *testing.T) {
	for _, c := range segmentationCases {
		if got := strings.Join(Words(c.input), "|"); got != c.want {
			t.Errorf("%s: Words(%q) = %q, want %q", c.name, c.input, got, c.want)
		}
	}
}
