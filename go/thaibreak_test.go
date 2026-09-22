package thaibreak

import (
	"reflect"
	"strings"
	"testing"
)

func TestTokenize(t *testing.T) {
	words := Words("ฉันรักภาษาไทย")
	expected := []string{"ฉัน", "รัก", "ภาษา", "ไทย"}
	if !reflect.DeepEqual(words, expected) {
		t.Errorf("Expected %v, got %v", expected, words)
	}

	joined := Join("สวัสดีครับ", "|")
	if joined != "สวัสดี|ครับ" {
		t.Errorf("Expected สวัสดี|ครับ, got %s", joined)
	}
}

func TestTypographicRules(t *testing.T) {
	text := "เดินเล่นๆ (ที่นี่ๆ) กรุงเทพฯ ฿100"
	lines := Lines(text, false)

	if strings.Contains(lines, DefaultBreakMarker+"ๆ") {
		t.Error("Break marker should not precede Mai Yamok (ๆ)")
	}
	if strings.Contains(lines, DefaultBreakMarker+"ฯ") {
		t.Error("Break marker should not precede Paiyannoi (ฯ)")
	}
	if strings.Contains(lines, "("+DefaultBreakMarker) {
		t.Error("Break marker should not follow opening bracket")
	}
	if strings.Contains(lines, DefaultBreakMarker+")") {
		t.Error("Break marker should not precede closing bracket")
	}
	if strings.Contains(lines, " "+DefaultBreakMarker) || strings.Contains(lines, DefaultBreakMarker+" ") {
		t.Error("Break marker should not be adjacent to space")
	}
}

func TestHtmlPreservation(t *testing.T) {
	html := "<div class=\"title\"><b>สวัสดี</b> &amp; ประเทศไทย</div>"
	broken := Lines(html, true)

	if !strings.Contains(broken, "<div class=\"title\">") {
		t.Error("Opening tag damaged")
	}
	if !strings.Contains(broken, "&amp;") {
		t.Error("HTML entity damaged")
	}
	if strings.ReplaceAll(broken, DefaultBreakMarker, "") != html {
		t.Error("Stripped HTML does not match original")
	}
}

func TestDisplayWidth(t *testing.T) {
	if w := DisplayWidth("ก"); w != 1 {
		t.Errorf("Expected 1, got %d", w)
	}
	// ที่ = ท (1) + ี (0) + ่ (0) = 1
	if w := DisplayWidth("ที่"); w != 1 {
		t.Errorf("Expected 1 for ที่, got %d", w)
	}
	// น้ำ = น (1) + ้ (0) + ำ (1) = 2
	if w := DisplayWidth("น้ำ"); w != 2 {
		t.Errorf("Expected 2 for น้ำ, got %d", w)
	}
}

func TestWrap(t *testing.T) {
	text := "บริษัท แอดวานซ์ อินโฟร์ เซอร์วิส จำกัด (มหาชน) ได้รายงานผลการดำเนินงานประจำปี 2567 มียอดขายรวม ฿180,000 ล้านบาท"
	wrapped := Wrap(text, 30)

	lines := strings.Split(wrapped, "\n")
	if len(lines) < 2 {
		t.Errorf("Expected multiple lines, got %d", len(lines))
	}
	for i, line := range lines {
		w := DisplayWidth(line)
		if w > 35 {
			t.Errorf("Line %d width %d exceeds margin: %s", i+1, w, line)
		}
	}
}

func BenchmarkTokenize(b *testing.B) {
	text := "บริษัท แอดวานซ์ อินโฟร์ เซอร์วิส จำกัด (มหาชน) ได้ประกาศผลประกอบการประจำปี 2567 มีรายได้รวม 180,000 ล้านบาท"
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = Words(text)
	}
}
