package thaibreak

import (
	"reflect"
	"strings"
	"testing"
)

// lineBreakCases maps input to expected output with '|' as the break marker.
var lineBreakCases = []struct{ name, input, want string }{
	{"LB9 combining mark stays with its base", "สวัสดี́ครับ", "สวัสดี́|ครับ"},
	{"LB21 no break before a hyphen", "สี-ขาว", "สี-|ขาว"},
	{"LB21 no break before an en dash (BA)", "ข้อความ–ข้อความ", "ข้อความ–|ข้อความ"},
	{"LB13 no break before CJK closing punctuation (CL)", "ไทย、ไทย", "ไทย、|ไทย"},
	{"LB13 no break before ? (EX)", "ไปไหม?ไปสิ", "ไป|ไหม?|ไป|สิ"},
	{"LB13 no break before a solidus (SY)", "ISO/IEC 29110", "ISO/|IEC 29110"},
	{"LB25 no break inside a date", "วันที่ 1/2/2567 นะ", "วัน|ที่ 1/2/2567 นะ"},
	{"LB25 no break inside a time", "เวลา 10:30 น.", "เวลา 10:30 น."},
	{"LB25 no break inside a range or a signed number", "ช่วง 10-20 คน ลบ -5 องศา", "ช่วง 10-20 คน ลบ -5 องศา"},
	{"LB25 no break after a prefix or before a postfix", "ราคา $(5) ลด 40%", "ราคา $(5) ลด 40%"},
	{"LB25 no break between IS and NU", "พ.ศ.2567", "พ.ศ.2567"},
}

func TestInsertLineBreaksUAX14(t *testing.T) {
	for _, c := range lineBreakCases {
		if got := GetDefaultLineBreaker().InsertLineBreaks(c.input, "|", false); got != c.want {
			t.Errorf("%s: InsertLineBreaks(%q) = %q, want %q", c.name, c.input, got, c.want)
		}
	}
}

// wrapCases maps input and width to the expected lines.
var wrapCases = []struct {
	name, input string
	width       int
	want        []string
}{
	{"an opening bracket never ends a line", "ประชาชน (ทั่วประเทศ) ไป", 9, []string{"ประชาชน", "(ทั่ว", "ประเทศ)", "ไป"}},
	{"a dash never starts a line", "ภาษาไทย–อังกฤษ", 7, []string{"ภาษา", "ไทย–", "อังกฤษ"}},
	{"mai yamok never starts a line", "ทดสอบเด็กๆๆๆๆๆๆ", 5, []string{"ทดสอบ", "เด็กๆๆๆๆๆๆ"}},
	{"an opening quote never ends a line", "สวัสดีครับ “ท่านผู้ชม”", 11, []string{"สวัสดีครับ", "“ท่านผู้ชม”"}},
	{"indentation that does not fit is dropped", "  ย่อหน้า ใหม่ ครับ", 6, []string{"ย่อหน้า", "ใหม่", "ครับ"}},
}

func TestWrapUAX14(t *testing.T) {
	for _, c := range wrapCases {
		got := strings.Split(GetDefaultLineBreaker().Wrap(c.input, c.width, "\n", false), "\n")
		if !reflect.DeepEqual(got, c.want) {
			t.Errorf("%s: Wrap(%q, %d) = %q, want %q", c.name, c.input, c.width, got, c.want)
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
