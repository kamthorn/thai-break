package thaibreak

import (
	"regexp"
	"strings"
	"unicode"
)

const (
	// DefaultBreakMarker is Zero-Width Space (U+200B).
	DefaultBreakMarker = "\u200B"
)

var (
	reNoBreakAfter  = regexp.MustCompile(`^[(\[{\\"“‘<«฿$€¥£#@（【《]$`)
	reNoBreakBefore = regexp.MustCompile(`^(?:[)\]}\\"”’>»,.:;!?/ๆฯ๏）】》]|ฯลฯ)$`)
	reHtmlTags      = regexp.MustCompile(`(?si:(<!--.*?-->|<script\b[^>]*>.*?</script>|<style\b[^>]*>.*?</style>|<[^>]+>|&[a-zA-Z0-9#]+;))`)
	reThaiCombining = regexp.MustCompile("[\u0E31\u0E34-\u0E3A\u0E47-\u0E4E\u200B]")
)

// LineBreaker handles Thai typographic line breaking and text wrapping.
type LineBreaker struct {
	tokenizer *Tokenizer
}

// NewLineBreaker creates a new LineBreaker bound to a Tokenizer.
func NewLineBreaker(tok *Tokenizer) *LineBreaker {
	return &LineBreaker{tokenizer: tok}
}

// CanBreakBetween checks if a break is permissible between left and right tokens.
// The tokens are treated as separate dictionary words, so a Thai|Thai junction
// is a word boundary.
func CanBreakBetween(left, right string) bool {
	if left == "" || right == "" {
		return false
	}

	cps := []rune(left + right)
	at := len([]rune(left))
	dict := make([]bool, len(cps)+1)
	dict[at] = true

	return lbBreakOpportunities(cps, dict)[at] == lbAllowed && passesTypographicRules(left, right)
}

// passesTypographicRules applies legacy token-level rules that are not yet
// expressed as UAX #14 rules.
func passesTypographicRules(left, right string) bool {
	leftRunes := []rune(left)
	rightRunes := []rune(right)

	// Whitespace safety
	if unicode.IsSpace(leftRunes[len(leftRunes)-1]) || unicode.IsSpace(rightRunes[0]) {
		return false
	}

	// No break after opening symbols
	if reNoBreakAfter.MatchString(left) {
		return false
	}

	// No break before closing symbols, the solidus (UAX #14 LB13), postfixes (ๆ, ฯ)
	if reNoBreakBefore.MatchString(right) {
		return false
	}

	// Latin letters and digits (UAX #14 LB23): WP01, ISO29110, 3rd
	l, r := leftRunes[len(leftRunes)-1], rightRunes[0]
	if (isASCIILetter(l) && isASCIIDigit(r)) || (isASCIIDigit(l) && isASCIILetter(r)) {
		return false
	}

	return true
}

func isASCIILetter(r rune) bool { return (r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') }

func isASCIIDigit(r rune) bool { return r >= '0' && r <= '9' }

// ThaiDisplayWidth calculates the visual terminal/column width for Thai text.
// Combining above/below marks, tone marks, and ZWSP are counted as 0 width.
func ThaiDisplayWidth(text string) int {
	stripped := reThaiCombining.ReplaceAllString(text, "")
	width := 0
	for _, r := range stripped {
		if r > 0x1100 && (r >= 0x2E80 && r <= 0x9FFF || r >= 0xAC00 && r <= 0xD7AF || r >= 0xF900 && r <= 0xFAFF) {
			width += 2 // Fullwidth / CJK
		} else {
			width += 1
		}
	}
	return width
}

// InsertLineBreaks inserts break markers at safe typographic boundaries.
func (b *LineBreaker) InsertLineBreaks(text string, marker string, isHtml bool) string {
	if text == "" {
		return ""
	}
	if marker == "" {
		marker = DefaultBreakMarker
	}

	if isHtml || (strings.Contains(text, "<") && strings.Contains(text, ">")) {
		return b.processHtml(text, marker)
	}
	return b.processPlain(text, marker)
}

func (b *LineBreaker) processPlain(text string, marker string) string {
	tokens := b.tokenizer.Tokenize(text, true)
	n := len(tokens)
	if n <= 1 {
		return text
	}

	// Token ends are the dictionary word boundaries inside Thai runs
	cps := make([]rune, 0, len(text))
	ends := make([]int, n)
	for i, tok := range tokens {
		cps = append(cps, []rune(tok)...)
		ends[i] = len(cps)
	}
	dict := make([]bool, len(cps)+1)
	for _, end := range ends {
		dict[end] = true
	}
	actions := lbBreakOpportunities(cps, dict)

	var sb strings.Builder
	for i := 0; i < n; i++ {
		sb.WriteString(tokens[i])
		if i+1 < n && actions[ends[i]] == lbAllowed && passesTypographicRules(tokens[i], tokens[i+1]) {
			sb.WriteString(marker)
		}
	}
	return sb.String()
}

func (b *LineBreaker) processHtml(html string, marker string) string {
	parts := reHtmlTags.Split(html, -1)
	matches := reHtmlTags.FindAllString(html, -1)

	var sb strings.Builder
	for i, part := range parts {
		if part != "" {
			sb.WriteString(b.processPlain(part, marker))
		}
		if i < len(matches) {
			sb.WriteString(matches[i]) // preserve HTML tag or entity intact
		}
	}
	return sb.String()
}

// Wrap soft-wraps Thai text to fit within `width` visual columns.
func (b *LineBreaker) Wrap(text string, width int, breakSep string, cutLongWords bool) string {
	if text == "" || width <= 0 {
		return text
	}
	if breakSep == "" {
		breakSep = "\n"
	}

	paragraphs := strings.Split(strings.ReplaceAll(text, "\r\n", "\n"), "\n")
	var wrappedParagraphs []string

	for _, para := range paragraphs {
		if para == "" {
			wrappedParagraphs = append(wrappedParagraphs, "")
			continue
		}

		tokens := b.tokenizer.Tokenize(para, true)
		var lines []string
		curLine := ""
		curWidth := 0

		for _, tok := range tokens {
			if tok == "" {
				continue
			}

			w := ThaiDisplayWidth(tok)

			// Skip leading whitespace on new line
			if curLine == "" && strings.TrimSpace(tok) == "" {
				continue
			}

			if curWidth+w <= width {
				curLine += tok
				curWidth += w
			} else {
				if curLine != "" {
					// Hanging punctuation rule
					if reNoBreakBefore.MatchString(tok) && curWidth+w <= width+3 {
						curLine += tok
						curWidth += w
						continue
					}

					lines = append(lines, strings.TrimRightFunc(curLine, unicode.IsSpace))

					if strings.TrimSpace(tok) == "" {
						curLine = ""
						curWidth = 0
						continue
					}

					curLine = tok
					curWidth = w
				} else {
					lines = append(lines, tok)
					curLine = ""
					curWidth = 0
				}
			}
		}

		if curLine != "" {
			lines = append(lines, strings.TrimRightFunc(curLine, unicode.IsSpace))
		}

		wrappedParagraphs = append(wrappedParagraphs, strings.Join(lines, breakSep))
	}

	return strings.Join(wrappedParagraphs, breakSep)
}
