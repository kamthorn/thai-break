//! Unicode Line Breaking Algorithm (UAX #14) for Unicode 16.0.
//!
//! Classes are resolved as in LB1 (AI, SG, XX → AL; CJ → NS), except that
//! Complex_Context (SA) is kept as `SA` (letters) and `SM` (combining marks) so
//! Thai runs can be segmented with the dictionary. QU is split into `QI`
//! (\p{Pi}), `QF` (\p{Pf}) and plain `QU` for LB15a, LB15b and LB19.

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
