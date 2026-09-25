package thaibreak

import (
	"reflect"
	"testing"
)

// lineBreakCases maps input to expected output with '|' as the break marker.
var lineBreakCases = []struct{ name, input, want string }{
	{"LB9 combining mark stays with its base", "สวัสดี́ครับ", "สวัสดี́|ครับ"},
	{"LB21 no break before a hyphen", "สี-ขาว", "สี-|ขาว"},
	{"LB21 no break before an en dash (BA)", "ข้อความ–ข้อความ", "ข้อความ–|ข้อความ"},
}

func TestInsertLineBreaksUAX14(t *testing.T) {
	for _, c := range lineBreakCases {
		if got := GetDefaultLineBreaker().InsertLineBreaks(c.input, "|", false); got != c.want {
			t.Errorf("%s: InsertLineBreaks(%q) = %q, want %q", c.name, c.input, got, c.want)
		}
	}
}

func TestMandatoryBreaks(t *testing.T) {
	got := lbBreakOpportunities([]rune("a\r\nb"), nil)
	want := []uint8{lbNoBreak, lbNoBreak, lbNoBreak, lbMandatory, lbMandatory}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("got %v, want %v", got, want)
	}
}
