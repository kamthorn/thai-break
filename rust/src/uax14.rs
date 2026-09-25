//! Unicode Line Breaking Algorithm (UAX #14) for Unicode 16.0.
//!
//! Classes are resolved as in LB1 (AI, SG, XX → AL; CJ → NS), except that
//! Complex_Context (SA) is kept as `SA` (letters) and `SM` (combining marks) so
//! Thai runs can be segmented with the dictionary. QU is split into `QI`
//! (\p{Pi}), `QF` (\p{Pf}) and plain `QU` for LB15a, LB15b and LB19. `HH` is
//! not assigned to any code point in Unicode 16.0 (U+2010 is BA, see LB20a).

// Rule code uses the class names of UAX #14 (A, B, K) for readability, and
// the class constants mirror the full table even where no rule refers to them.
#![allow(non_snake_case, dead_code)]

use crate::linebreak_data::{LB_STARTS, LB_VALUES};

pub(crate) const AL: u8 = 0;
pub(crate) const BK: u8 = 1;
pub(crate) const CR: u8 = 2;
pub(crate) const LF: u8 = 3;
pub(crate) const NL: u8 = 4;
pub(crate) const SP: u8 = 5;
pub(crate) const ZW: u8 = 6;
pub(crate) const WJ: u8 = 7;
pub(crate) const GL: u8 = 8;
pub(crate) const CL: u8 = 9;
pub(crate) const CP: u8 = 10;
pub(crate) const EX: u8 = 11;
pub(crate) const IS: u8 = 12;
pub(crate) const SY: u8 = 13;
pub(crate) const OP: u8 = 14;
pub(crate) const QU: u8 = 15;
pub(crate) const QI: u8 = 16;
pub(crate) const QF: u8 = 17;
pub(crate) const NS: u8 = 18;
pub(crate) const B2: u8 = 19;
pub(crate) const BA: u8 = 20;
pub(crate) const BB: u8 = 21;
pub(crate) const HY: u8 = 22;
pub(crate) const HH: u8 = 23;
pub(crate) const CB: u8 = 24;
pub(crate) const IN: u8 = 25;
pub(crate) const HL: u8 = 26;
pub(crate) const NU: u8 = 27;
pub(crate) const PR: u8 = 28;
pub(crate) const PO: u8 = 29;
pub(crate) const ID: u8 = 30;
pub(crate) const EB: u8 = 31;
pub(crate) const EM: u8 = 32;
pub(crate) const H2: u8 = 33;
pub(crate) const H3: u8 = 34;
pub(crate) const JL: u8 = 35;
pub(crate) const JV: u8 = 36;
pub(crate) const JT: u8 = 37;
pub(crate) const RI: u8 = 38;
pub(crate) const ZWJ: u8 = 39;
pub(crate) const CM: u8 = 40;
pub(crate) const AK: u8 = 41;
pub(crate) const AP: u8 = 42;
pub(crate) const AS: u8 = 43;
pub(crate) const VF: u8 = 44;
pub(crate) const VI: u8 = 45;
pub(crate) const SA: u8 = 46;
pub(crate) const SM: u8 = 47;

const CLASS_MASK: u8 = 0x3F;
const FLAG_EAST_ASIAN: u8 = 0x40;
const FLAG_EXT_PICT_UNASSIGNED: u8 = 0x80;

/// Packed table value (class id and flags) for a code point.
pub(crate) fn lb_value(ch: char) -> u8 {
    let cp = ch as u32;
    let idx = LB_STARTS.partition_point(|&start| start <= cp) - 1;
    LB_VALUES[idx]
}

/// Resolved Line_Break class of a code point.
pub(crate) fn lb_class(ch: char) -> u8 {
    lb_value(ch) & CLASS_MASK
}

/// Whether East_Asian_Width is F, W or H (the $EastAsian set of LB19a and LB30).
pub(crate) fn is_east_asian(ch: char) -> bool {
    lb_value(ch) & FLAG_EAST_ASIAN != 0
}

/// Whether the code point is Extended_Pictographic and unassigned (LB30b).
pub(crate) fn is_ext_pict_unassigned(ch: char) -> bool {
    lb_value(ch) & FLAG_EXT_PICT_UNASSIGNED != 0
}

/// Break actions returned by [`break_opportunities`].
pub(crate) const NO_BREAK: u8 = 0;
pub(crate) const ALLOWED: u8 = 1;
pub(crate) const MANDATORY: u8 = 2;

/// Class of a position outside the text (sot / eot).
const NONE: u8 = 0xFF;

const THAI_MAIYAMOK: char = '\u{0E46}';
const THAI_PAIYANNOI: char = '\u{0E2F}';
const HYPHEN: char = '\u{2010}';
const DOTTED_CIRCLE: char = '\u{25CC}';

/// A character with its attached combining marks (LB9).
struct Unit {
    cls: u8,
    cp: char,
    start: usize,
    ea: bool,
    xp: bool,
    zwj: bool,
}

/// Compute the break action before every position of `cps`.
///
/// Between two SA letters (Thai, Lao, Khmer, Myanmar) the dictionary decides:
/// a break is allowed where `dict_breaks[i]` is true. As a Thai tailoring,
/// there is never a break before ๆ (U+0E46) or ฯ (U+0E2F). When `dict_breaks`
/// is `None`, SA is resolved to AL as in LB1 and no tailoring is applied.
///
/// The result has `cps.len() + 1` entries: position 0 is never a break (LB2)
/// and position `cps.len()` always is (LB3).
pub(crate) fn break_opportunities(cps: &[char], dict_breaks: Option<&[bool]>) -> Vec<u8> {
    let n = cps.len();
    let mut actions = vec![NO_BREAK; n + 1];
    if n == 0 {
        return actions;
    }
    actions[n] = MANDATORY;

    // LB9/LB10: group X (CM|ZWJ)* into units that take the class of X.
    let mut units: Vec<Unit> = Vec::with_capacity(n);
    for (i, &cp) in cps.iter().enumerate() {
        let v = lb_value(cp);
        let mut cls = v & CLASS_MASK;
        if cls == SM {
            cls = CM;
        }
        if cls == CM || cls == ZWJ {
            if let Some(last) = units.last_mut() {
                if ![BK, CR, LF, NL, SP, ZW].contains(&last.cls) {
                    last.zwj = cls == ZWJ;
                    continue;
                }
            }
        }
        let resolved = match cls {
            CM | ZWJ => AL,
            SA if dict_breaks.is_none() => AL,
            _ => cls,
        };
        units.push(Unit {
            cls: resolved,
            cp,
            start: i,
            ea: v & FLAG_EAST_ASIAN != 0,
            xp: v & FLAG_EXT_PICT_UNASSIGNED != 0,
            zwj: cls == ZWJ,
        });
    }

    for b in 1..units.len() {
        actions[units[b].start] = pair_action(&units, b, dict_breaks);
    }
    actions
}

/// Apply rules LB4–LB31 in order to the boundary before unit `b`.
fn pair_action(units: &[Unit], b: usize, dict_breaks: Option<&[bool]>) -> u8 {
    let a = b - 1;
    let count = units.len();
    let cls = |i: isize| -> u8 {
        if i < 0 || i as usize >= count {
            return NONE;
        }
        match units[i as usize].cls {
            SA => AL,
            c => c,
        }
    };
    let (ai, bi) = (a as isize, b as isize);
    let (mut A, mut B) = (units[a].cls, units[b].cls);

    // LB4–LB6: mandatory breaks
    if A == BK {
        return MANDATORY;
    }
    if A == CR && B == LF {
        return NO_BREAK;
    }
    if A == CR || A == LF || A == NL {
        return MANDATORY;
    }
    if [BK, CR, LF, NL].contains(&B) {
        return NO_BREAK;
    }
    // LB7: × SP, × ZW
    if B == SP || B == ZW {
        return NO_BREAK;
    }
    // Last unit before the boundary, skipping spaces (for the "X SP*" rules)
    let mut k = a;
    while k > 0 && units[k].cls == SP {
        k -= 1;
    }
    let K = cls(k as isize);
    // LB8: ZW SP* ÷
    if K == ZW {
        return ALLOWED;
    }
    // LB8a: ZWJ ×
    if units[a].zwj {
        return NO_BREAK;
    }
    // SA resolution (LB1), Thai tailoring: ๆ and ฯ never start a line, even after spaces
    if dict_breaks.is_some() && (units[b].cp == THAI_MAIYAMOK || units[b].cp == THAI_PAIYANNOI) {
        return NO_BREAK;
    }
    // SA resolution (LB1): the dictionary decides inside SA runs
    if A == SA && B == SA {
        let allowed = dict_breaks.and_then(|d| d.get(units[b].start)).copied().unwrap_or(false);
        return if allowed { ALLOWED } else { NO_BREAK };
    }
    A = cls(ai);
    B = cls(bi);
    // LB11: × WJ, WJ ×
    if A == WJ || B == WJ {
        return NO_BREAK;
    }
    // LB12: GL ×
    if A == GL {
        return NO_BREAK;
    }
    // LB12a: [^SP BA HY] × GL
    if B == GL && ![SP, BA, HY].contains(&A) {
        return NO_BREAK;
    }
    // LB13: × CL, × CP, × EX, × SY
    if [CL, CP, EX, SY].contains(&B) {
        return NO_BREAK;
    }
    // LB14: OP SP* ×
    if K == OP {
        return NO_BREAK;
    }
    // LB15a: (sot | BK | CR | LF | NL | OP | QU | GL | SP | ZW) [\p{Pi}&QU] SP* ×
    if K == QI && (k == 0 || [BK, CR, LF, NL, OP, QU, QI, QF, GL, SP, ZW].contains(&cls(k as isize - 1))) {
        return NO_BREAK;
    }
    // LB15b: × [\p{Pf}&QU] (SP | GL | WJ | CL | QU | CP | EX | IS | SY | BK | CR | LF | NL | ZW | eot)
    if B == QF && (b + 1 == count || [SP, GL, WJ, CL, QU, QI, QF, CP, EX, IS, SY, BK, CR, LF, NL, ZW].contains(&cls(bi + 1))) {
        return NO_BREAK;
    }
    // LB15c: SP ÷ IS NU
    if A == SP && B == IS && cls(bi + 1) == NU {
        return ALLOWED;
    }
    // LB15d: × IS
    if B == IS {
        return NO_BREAK;
    }
    // LB16: (CL | CP) SP* × NS
    if B == NS && (K == CL || K == CP) {
        return NO_BREAK;
    }
    // LB17: B2 SP* × B2
    if B == B2 && K == B2 {
        return NO_BREAK;
    }
    // LB18: SP ÷
    if A == SP {
        return ALLOWED;
    }
    // LB19: × [QU - \p{Pi}], [QU - \p{Pf}] ×
    if B == QU || B == QF || A == QU || A == QI {
        return NO_BREAK;
    }
    // LB19a: quotation marks bind unless both neighbours are East Asian
    if is_quote(B) && (!units[a].ea || b + 1 == count || !units[b + 1].ea) {
        return NO_BREAK;
    }
    if is_quote(A) && (!units[b].ea || a == 0 || !units[a - 1].ea) {
        return NO_BREAK;
    }
    // LB20: ÷ CB, CB ÷
    if A == CB || B == CB {
        return ALLOWED;
    }
    // LB20a: (sot | BK | CR | LF | NL | SP | ZW | CB | GL) (HY | [‐]) × AL
    if (A == HY || units[a].cp == HYPHEN) && B == AL
        && (a == 0 || [BK, CR, LF, NL, SP, ZW, CB, GL].contains(&cls(ai - 1)))
    {
        return NO_BREAK;
    }
    // LB21: × BA, × HY, × NS, BB ×
    if [BA, HY, NS].contains(&B) || A == BB {
        return NO_BREAK;
    }
    // LB21a: HL (HY | [BA - $EastAsian]) × [^HL]
    if (A == HY || (A == BA && !units[a].ea)) && cls(ai - 1) == HL && B != HL {
        return NO_BREAK;
    }
    // LB21b: SY × HL
    if A == SY && B == HL {
        return NO_BREAK;
    }
    // LB22: × IN
    if B == IN {
        return NO_BREAK;
    }
    // LB23: (AL | HL) × NU, NU × (AL | HL)
    if (is_alpha(A) && B == NU) || (A == NU && is_alpha(B)) {
        return NO_BREAK;
    }
    // LB23a: PR × (ID | EB | EM), (ID | EB | EM) × PO
    if (A == PR && [ID, EB, EM].contains(&B)) || ([ID, EB, EM].contains(&A) && B == PO) {
        return NO_BREAK;
    }
    // LB24: (PR | PO) × (AL | HL), (AL | HL) × (PR | PO)
    if ((A == PR || A == PO) && is_alpha(B)) || (is_alpha(A) && (B == PR || B == PO)) {
        return NO_BREAK;
    }
    // LB25: numbers
    if B == PO || B == PR {
        // NU (SY | IS)* (CL | CP)? × (PO | PR)
        let mut j = if A == CL || A == CP { ai - 1 } else { ai };
        while cls(j) == SY || cls(j) == IS {
            j -= 1;
        }
        if cls(j) == NU {
            return NO_BREAK;
        }
    }
    if (A == PO || A == PR) && B == OP && (cls(bi + 1) == NU || (cls(bi + 1) == IS && cls(bi + 2) == NU)) {
        return NO_BREAK; // (PO | PR) × OP IS? NU
    }
    if B == NU && [PO, PR, HY, IS].contains(&A) {
        return NO_BREAK; // (PO | PR | HY | IS) × NU
    }
    if B == NU {
        // NU (SY | IS)* × NU
        let mut j = ai;
        while cls(j) == SY || cls(j) == IS {
            j -= 1;
        }
        if cls(j) == NU {
            return NO_BREAK;
        }
    }
    // LB26: Korean syllable blocks
    if (A == JL && [JL, JV, H2, H3].contains(&B))
        || ((A == JV || A == H2) && (B == JV || B == JT))
        || ((A == JT || A == H3) && B == JT)
    {
        return NO_BREAK;
    }
    // LB27: (JL | JV | JT | H2 | H3) × PO, PR × (JL | JV | JT | H2 | H3)
    if (is_hangul(A) && B == PO) || (A == PR && is_hangul(B)) {
        return NO_BREAK;
    }
    // LB28: (AL | HL) × (AL | HL)
    if is_alpha(A) && is_alpha(B) {
        return NO_BREAK;
    }
    // LB28a: Brahmic orthographic syllables
    if in_aksara(units, ai, bi, &cls) {
        return NO_BREAK;
    }
    // LB29: IS × (AL | HL)
    if A == IS && is_alpha(B) {
        return NO_BREAK;
    }
    // LB30: (AL | HL | NU) × [OP - $EastAsian], [CP - $EastAsian] × (AL | HL | NU)
    if ((is_alpha(A) || A == NU) && B == OP && !units[b].ea)
        || (A == CP && !units[a].ea && (is_alpha(B) || B == NU))
    {
        return NO_BREAK;
    }
    // LB30a: break between pairs of regional indicators only
    if A == RI && B == RI {
        let run = units[..=a].iter().rev().take_while(|u| u.cls == RI).count();
        if run % 2 == 1 {
            return NO_BREAK;
        }
    }
    // LB30b: EB × EM, [\p{Extended_Pictographic}&\p{Cn}] × EM
    if B == EM && (A == EB || units[a].xp) {
        return NO_BREAK;
    }
    // LB31: ÷
    ALLOWED
}

/// LB28a for the boundary between units `a` and `b`.
fn in_aksara(units: &[Unit], a: isize, b: isize, cls: &dyn Fn(isize) -> u8) -> bool {
    let is_dotted = |i: isize| i >= 0 && (i as usize) < units.len() && units[i as usize].cp == DOTTED_CIRCLE;
    let aksara = |i: isize| cls(i) == AK || cls(i) == AS || is_dotted(i);
    let (A, B) = (cls(a), cls(b));
    (A == AP && aksara(b))
        || (aksara(a) && (B == VF || B == VI))
        || (A == VI && aksara(a - 1) && (B == AK || is_dotted(b)))
        || (aksara(a) && aksara(b) && cls(b + 1) == VF)
}

fn is_alpha(c: u8) -> bool {
    c == AL || c == HL
}

fn is_quote(c: u8) -> bool {
    c == QU || c == QI || c == QF
}

fn is_hangul(c: u8) -> bool {
    [JL, JV, JT, H2, H3].contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_break_classes() {
        let expected = [
            ('a', AL), ('1', NU), (' ', SP), ('-', HY), ('–', BA),
            ('(', OP), (')', CP), ('}', CL), ('、', CL), ('/', SY),
            ('.', IS), ('!', EX), ('"', QU), ('“', QI), ('”', QF),
            ('$', PR), ('฿', PR), ('\\', PR), ('%', PO), ('#', AL),
            ('ก', SA), ('ๆ', SA), ('ั', SM), ('่', SM), ('๐', NU),
            ('๚', BA), ('中', ID), ('가', H2), ('א', HL),
            ('\u{200B}', ZW), ('\u{00A0}', GL), ('\u{200D}', ZWJ), ('\u{0301}', CM),
            ('😀', ID), ('\u{1F1F9}', RI), ('\u{10FFFF}', AL),
        ];
        for (ch, class) in expected {
            assert_eq!(lb_class(ch), class, "class of U+{:04X}", ch as u32);
        }
    }

    #[test]
    fn flags() {
        assert!(is_east_asian('（'));
        assert!(!is_east_asian('('));
        assert!(is_ext_pict_unassigned('\u{1FFFD}'));
        assert!(!is_ext_pict_unassigned('😀'));
    }
}

#[cfg(test)]
mod engine_tests {
    use super::*;

    /// Official conformance test (SA resolved to AL, no dictionary).
    #[test]
    fn line_break_test_conformance() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../testdata/LineBreakTest-16.0.0.txt");
        let Ok(data) = std::fs::read_to_string(path) else {
            eprintln!("LineBreakTest data not found, skipping");
            return;
        };
        let mut failures = Vec::new();
        for line in data.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut cps = Vec::new();
            let mut expected = Vec::new();
            for field in line.split_whitespace() {
                match field {
                    "×" => expected.push(false),
                    "÷" => expected.push(true),
                    hex => {
                        let cp = u32::from_str_radix(hex, 16).unwrap();
                        // Surrogates cannot be a char; they resolve to AL like U+FFFD (AI)
                        cps.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                    }
                }
            }
            let actual = break_opportunities(&cps, None);
            if expected.iter().enumerate().any(|(i, &e)| (actual[i] != NO_BREAK) != e) {
                failures.push(line);
            }
        }
        assert!(failures.is_empty(), "{} LineBreakTest cases failed: {:?}", failures.len(), &failures[..failures.len().min(10)]);
    }

    #[test]
    fn mandatory_breaks() {
        let cps: Vec<char> = "a\r\nb".chars().collect();
        assert_eq!(
            break_opportunities(&cps, None),
            vec![NO_BREAK, NO_BREAK, NO_BREAK, MANDATORY, MANDATORY]
        );
    }
}
