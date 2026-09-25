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
	{"ties keep the earlier word whole", "บอกว่าอึดอัด", "บอก|ว่า|อึดอัด"},
	{"ties keep the earlier word whole (2)", "ลาออกจากรองประธาน", "ลาออก|จาก|รอง|ประธาน"},
}

func TestLongTextIsSegmentedToTheEnd(t *testing.T) {
	// Longer than the 50,000-edge limit that used to leave the rest of the text as one token
	sentence := "การประชุมสามัญผู้ถือหุ้นประจำปีจัดขึ้นที่โรงแรมในกรุงเทพมหานคร"
	words := Words(strings.Repeat(sentence, 2000))
	if want := 2000 * len(Words(sentence)); len(words) != want {
		t.Fatalf("got %d words, want %d", len(words), want)
	}
	for _, w := range words {
		if len([]rune(w)) >= 20 {
			t.Fatalf("unexpected long token of %d characters", len([]rune(w)))
		}
	}
}

func TestSegmentation(t *testing.T) {
	for _, c := range segmentationCases {
		if got := strings.Join(Words(c.input), "|"); got != c.want {
			t.Errorf("%s: Words(%q) = %q, want %q", c.name, c.input, got, c.want)
		}
	}
}
