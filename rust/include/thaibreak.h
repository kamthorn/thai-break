#ifndef THAIBREAK_H
#define THAIBREAK_H

/**
 * ThaiBreak — Fast, high-accuracy Thai word segmentation and typographic line breaker.
 * 
 * C / C++ Header
 * License: Apache-2.0
 */

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Initialize ThaiBreak with dictionary and bigrams file paths.
 * @param dict_path Path to wordlist.tsv
 * @param bigram_path Path to bigrams.tsv (optional, pass NULL if not used)
 * @return 0 on success, -1 on error
 */
int thaibreak_init(const char *dict_path, const char *bigram_path);

/**
 * Tokenize UTF-8 Thai text into word tokens.
 * @param text UTF-8 encoded text
 * @param count Pointer to receive the number of tokens
 * @return Array of null-terminated UTF-8 strings. Free with thaibreak_free_tokens.
 */
char **thaibreak_tokenize(const char *text, size_t *count);

/**
 * Free token array returned by thaibreak_tokenize.
 * @param tokens Pointer returned by thaibreak_tokenize
 * @param count Number of tokens in the array
 */
void thaibreak_free_tokens(char **tokens, size_t count);

/**
 * Insert break opportunities into text.
 * @param text UTF-8 encoded text
 * @param marker Break marker to insert (e.g. "\u200B" for ZWSP). Pass NULL for default ZWSP.
 * @param is_html 1 if text contains HTML (preserves tags & entities), 0 for plain text
 * @return Newly allocated UTF-8 string. Free with thaibreak_free_string.
 */
char *thaibreak_lines(const char *text, const char *marker, int is_html);

/**
 * Hard-wrap text into lines of maximum visual display width.
 * @param text UTF-8 encoded text
 * @param width Maximum display width (columns)
 * @param is_html 1 for HTML mode, 0 for plain text
 * @return Newly allocated UTF-8 string with \n line breaks. Free with thaibreak_free_string.
 */
char *thaibreak_wrap(const char *text, size_t width, int is_html);

/**
 * Calculate visual terminal/column display width of Thai text.
 * Thai tone marks and upper/lower vowels are counted as 0 width.
 * @param text UTF-8 encoded text
 * @return Visual display width
 */
size_t thaibreak_display_width(const char *text);

/**
 * Free string returned by thaibreak_lines or thaibreak_wrap.
 * @param s Pointer to string to free
 */
void thaibreak_free_string(char *s);

#ifdef __cplusplus
}
#endif

#endif /* THAIBREAK_H */
