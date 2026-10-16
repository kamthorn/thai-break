package thaibreak

import (
	"reflect"
	"regexp"
	"strings"
	"testing"
	"time"
)

// The regex that markupSpans replaces
var reHtmlTags = regexp.MustCompile(`(?si:(<!--.*?-->|<script\b[^>]*>.*?</script>|<style\b[^>]*>.*?</style>|<[a-zA-Z/!?][^>]*>|&[a-zA-Z0-9#]+;))`)

func TestMarkupSpansMatchesRegex(t *testing.T) {
	pieces := []string{
		"<", ">", "!", "-", "/", "?", "<!--", "-->", "<script", "</script>", "<SCRIPT", "</Script>",
		"<style", "</STYLE>", "<scripts", "<script_", " ", "\n", "a", "Z", "&", "#", ";", "&amp;", "ก", "ไทย",
		"<b>", "</b>", "<ab",
	}
	seed := uint64(0x9E3779B97F4A7C15)
	next := func() uint64 {
		seed ^= seed << 13
		seed ^= seed >> 7
		seed ^= seed << 17
		return seed
	}
	for n := 0; n < 20000; n++ {
		var sb strings.Builder
		for k := next() % 12; k > 0; k-- {
			sb.WriteString(pieces[next()%uint64(len(pieces))])
		}
		text := sb.String()
		var want [][2]int
		for _, m := range reHtmlTags.FindAllStringIndex(text, -1) {
			want = append(want, [2]int{m[0], m[1]})
		}
		if got := markupSpans(text); !reflect.DeepEqual(got, want) {
			t.Fatalf("markupSpans(%q) = %v, want %v", text, got, want)
		}
	}
}

func TestMarkupSpansUnterminatedIsLinear(t *testing.T) {
	for _, piece := range []string{"<script>", "<style x>", "<!-->", "<a", "<!--<script>"} {
		text := strings.Repeat(piece, 200000)
		start := time.Now()
		markupSpans(text)
		if d := time.Since(start); d > 2*time.Second {
			t.Errorf("%q took %v", piece, d)
		}
	}
}
