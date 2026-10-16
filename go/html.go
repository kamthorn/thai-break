package thaibreak

import "strings"

// terminator finds the next position of a fixed string, remembering the last
// lookup. Lookups come from increasing positions, so a cached hit at or after
// the new start is still the first one and a cached miss stays a miss: every
// byte is searched at most once.
type terminator struct {
	needle string
	pos    int // -1: not found
	known  bool
}

func (t *terminator) find(lower string, from int) int {
	if t.known && (t.pos < 0 || t.pos >= from) {
		return t.pos
	}
	t.known = true
	if from > len(lower) {
		t.pos = -1
	} else if j := strings.Index(lower[from:], t.needle); j < 0 {
		t.pos = -1
	} else {
		t.pos = from + j
	}
	return t.pos
}

func isASCIIWordByte(c byte) bool {
	return c == '_' || c >= '0' && c <= '9' || c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z'
}

// markupSpans returns the byte ranges of the comments, script and style
// blocks, tags and entities in html. It finds the same spans as
// `(?si:<!--.*?-->|<script\b[^>]*>.*?</script>|<style\b[^>]*>.*?</style>|<[a-zA-Z/!?][^>]*>|&[a-zA-Z0-9#]+;)`
// but in linear time: with that regex, every `<script>`, `<!--` or `<a` whose
// terminator is missing scans to the end of the text again.
func markupSpans(html string) [][2]int {
	lower := asciiLower(html) // keeps byte offsets, unlike strings.ToLower
	commentEnd := &terminator{needle: "-->"}
	scriptEnd := &terminator{needle: "</script>"}
	styleEnd := &terminator{needle: "</style>"}
	tagEnd := &terminator{needle: ">"}

	var spans [][2]int
	n := len(html)
	for i := 0; i < n; {
		end := -1
		switch html[i] {
		case '<':
			if strings.HasPrefix(html[i:], "<!--") {
				if j := commentEnd.find(lower, i+4); j >= 0 {
					end = j + 3
				}
			}
			for _, raw := range []struct {
				open  string
				close *terminator
			}{{"<script", scriptEnd}, {"<style", styleEnd}} {
				after := i + len(raw.open)
				if end < 0 && strings.HasPrefix(lower[i:], raw.open) && (after == n || !isASCIIWordByte(html[after])) {
					if k := tagEnd.find(lower, after); k >= 0 {
						if j := raw.close.find(lower, k+1); j >= 0 {
							end = j + len(raw.close.needle)
						}
					}
				}
			}
			if end < 0 && i+1 < n {
				if c := html[i+1]; c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c == '/' || c == '!' || c == '?' {
					if k := tagEnd.find(lower, i+1); k >= 0 {
						end = k + 1
					}
				}
			}
		case '&':
			j := i + 1
			for j < n && (html[j] == '#' || html[j] >= '0' && html[j] <= '9' || html[j] >= 'a' && html[j] <= 'z' || html[j] >= 'A' && html[j] <= 'Z') {
				j++
			}
			if j > i+1 && j < n && html[j] == ';' {
				end = j + 1
			}
		}
		if end < 0 {
			i++
			continue
		}
		spans = append(spans, [2]int{i, end})
		i = end
	}
	return spans
}

func asciiLower(s string) string {
	b := []byte(s)
	for i, c := range b {
		if c >= 'A' && c <= 'Z' {
			b[i] = c + 'a' - 'A'
		}
	}
	return string(b)
}
