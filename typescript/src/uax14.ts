/**
 * Unicode Line Breaking Algorithm (UAX #14) for Unicode 16.0.
 *
 * Classes are resolved as in LB1 (AI, SG, XX → AL; CJ → NS), except that
 * Complex_Context (SA) is kept as SA (letters) and SM (combining marks) so
 * Thai runs can be segmented with the dictionary. QU is split into QI
 * (\p{Pi}), QF (\p{Pf}) and plain QU for LB15a, LB15b and LB19.
 */
import { LB_STARTS, LB_VALUES } from './linebreak-data.js';

export const enum LB {
  AL, BK, CR, LF, NL, SP, ZW, WJ, GL, CL, CP, EX, IS, SY, OP, QU, QI, QF, NS, B2,
  BA, BB, HY, HH, CB, IN, HL, NU, PR, PO, ID, EB, EM, H2, H3, JL, JV, JT, RI, ZWJ,
  CM, AK, AP, AS, VF, VI, SA, SM,
}

const CLASS_MASK = 0x3f;
const FLAG_EAST_ASIAN = 0x40;
const FLAG_EXT_PICT_UNASSIGNED = 0x80;

/** Packed table value (class id and flags) for a code point. */
export function lbValue(cp: number): number {
  let lo = 0;
  let hi = LB_STARTS.length - 1;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (LB_STARTS[mid] <= cp) {
      lo = mid;
    } else {
      hi = mid - 1;
    }
  }
  return LB_VALUES[lo];
}

/** Resolved Line_Break class of a code point. */
export function lbClass(cp: number): LB {
  return lbValue(cp) & CLASS_MASK;
}

/** Whether East_Asian_Width is F, W or H (the $EastAsian set of LB19a and LB30). */
export function isEastAsian(cp: number): boolean {
  return (lbValue(cp) & FLAG_EAST_ASIAN) !== 0;
}

/** Whether the code point is Extended_Pictographic and unassigned (LB30b). */
export function isExtPictUnassigned(cp: number): boolean {
  return (lbValue(cp) & FLAG_EXT_PICT_UNASSIGNED) !== 0;
}
