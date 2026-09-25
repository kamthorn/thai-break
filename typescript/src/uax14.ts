/**
 * Unicode Line Breaking Algorithm (UAX #14) for Unicode 16.0.
 *
 * Classes are resolved as in LB1 (AI, SG, XX → AL; CJ → NS), except that
 * Complex_Context (SA) is kept as SA (letters) and SM (combining marks) so
 * Thai runs can be segmented with the dictionary. QU is split into QI
 * (\p{Pi}), QF (\p{Pf}) and plain QU for LB15a, LB15b and LB19. HH is not
 * assigned to any code point in Unicode 16.0 (U+2010 is BA, see LB20a).
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

/** Break actions returned by breakOpportunities(). */
export const NO_BREAK = 0;
export const ALLOWED = 1;
export const MANDATORY = 2;

/** Class of a position outside the text (sot / eot). */
const NONE = -1;

const THAI_MAIYAMOK = 0x0e46;
const THAI_PAIYANNOI = 0x0e2f;
const HYPHEN = 0x2010;
const DOTTED_CIRCLE = 0x25cc;

/** A character with its attached combining marks (LB9). */
interface Unit {
  cls: number;
  cp: number;
  start: number;
  ea: boolean;
  xp: boolean;
  zwj: boolean;
}

/**
 * Compute the break action before every position of a code point sequence.
 *
 * Between two SA letters (Thai, Lao, Khmer, Myanmar) the dictionary decides:
 * a break is allowed where dictBreaks[i] is true. As a Thai tailoring, there
 * is never a break before ๆ (U+0E46) or ฯ (U+0E2F). When dictBreaks is null,
 * SA is resolved to AL as in LB1 and no tailoring is applied.
 *
 * Returns cps.length + 1 actions: position 0 is never a break (LB2) and
 * position cps.length always is (LB3).
 */
export function breakOpportunities(cps: number[], dictBreaks: boolean[] | null = null): number[] {
  const n = cps.length;
  const actions: number[] = new Array(n + 1).fill(NO_BREAK);
  if (n === 0) return actions;
  actions[n] = MANDATORY;

  // LB9/LB10: group X (CM|ZWJ)* into units that take the class of X.
  const units: Unit[] = [];
  for (let i = 0; i < n; i++) {
    const cp = cps[i];
    const v = lbValue(cp);
    let cls = v & CLASS_MASK;
    if (cls === LB.SM) cls = LB.CM;
    if (cls === LB.CM || cls === LB.ZWJ) {
      const last = units[units.length - 1];
      if (last && ![LB.BK, LB.CR, LB.LF, LB.NL, LB.SP, LB.ZW].includes(last.cls)) {
        last.zwj = cls === LB.ZWJ;
        continue;
      }
    }
    let resolved = cls;
    if (cls === LB.CM || cls === LB.ZWJ || (cls === LB.SA && dictBreaks === null)) {
      resolved = LB.AL;
    }
    units.push({
      cls: resolved,
      cp,
      start: i,
      ea: (v & FLAG_EAST_ASIAN) !== 0,
      xp: (v & FLAG_EXT_PICT_UNASSIGNED) !== 0,
      zwj: cls === LB.ZWJ,
    });
  }

  for (let b = 1; b < units.length; b++) {
    actions[units[b].start] = pairAction(units, b, dictBreaks);
  }
  return actions;
}

/** Apply rules LB4–LB31 in order to the boundary before unit b. */
function pairAction(units: Unit[], b: number, dictBreaks: boolean[] | null): number {
  const a = b - 1;
  const count = units.length;
  const cls = (i: number): number => {
    if (i < 0 || i >= count) return NONE;
    return units[i].cls === LB.SA ? LB.AL : units[i].cls;
  };
  let A = units[a].cls;
  let B = units[b].cls;

  // LB4–LB6: mandatory breaks
  if (A === LB.BK) {
    return MANDATORY;
  }
  if (A === LB.CR && B === LB.LF) {
    return NO_BREAK;
  }
  if (A === LB.CR || A === LB.LF || A === LB.NL) {
    return MANDATORY;
  }
  if ([LB.BK, LB.CR, LB.LF, LB.NL].includes(B)) {
    return NO_BREAK;
  }
  // LB7: × SP, × ZW
  if (B === LB.SP || B === LB.ZW) {
    return NO_BREAK;
  }
  // Last unit before the boundary, skipping spaces (for the "X SP*" rules)
  let k = a;
  while (k > 0 && units[k].cls === LB.SP) k--;
  const K = cls(k);
  // LB8: ZW SP* ÷
  if (K === LB.ZW) {
    return ALLOWED;
  }
  // LB8a: ZWJ ×
  if (units[a].zwj) {
    return NO_BREAK;
  }
  // SA resolution (LB1), Thai tailoring: ๆ and ฯ never start a line, even after spaces
  if (dictBreaks !== null && (units[b].cp === THAI_MAIYAMOK || units[b].cp === THAI_PAIYANNOI)) {
    return NO_BREAK;
  }
  // SA resolution (LB1): the dictionary decides inside SA runs
  if (A === LB.SA && B === LB.SA) {
    return dictBreaks?.[units[b].start] ? ALLOWED : NO_BREAK;
  }
  A = cls(a);
  B = cls(b);
  // LB13: × CL, × CP, × EX, × SY
  if ([LB.CL, LB.CP, LB.EX, LB.SY].includes(B)) {
    return NO_BREAK;
  }
  // LB15c: SP ÷ IS NU
  if (A === LB.SP && B === LB.IS && cls(b + 1) === LB.NU) {
    return ALLOWED;
  }
  // LB15d: × IS
  if (B === LB.IS) {
    return NO_BREAK;
  }
  // LB18: SP ÷
  if (A === LB.SP) {
    return ALLOWED;
  }
  // LB20a: (sot | BK | CR | LF | NL | SP | ZW | CB | GL) (HY | [‐]) × AL
  if ((A === LB.HY || units[a].cp === HYPHEN) && B === LB.AL &&
      (a === 0 || [LB.BK, LB.CR, LB.LF, LB.NL, LB.SP, LB.ZW, LB.CB, LB.GL].includes(cls(a - 1)))) {
    return NO_BREAK;
  }
  // LB21: × BA, × HY, × NS, BB ×
  if ([LB.BA, LB.HY, LB.NS].includes(B) || A === LB.BB) {
    return NO_BREAK;
  }
  // LB21a: HL (HY | [BA - $EastAsian]) × [^HL]
  if ((A === LB.HY || (A === LB.BA && !units[a].ea)) && cls(a - 1) === LB.HL && B !== LB.HL) {
    return NO_BREAK;
  }
  // LB21b: SY × HL
  if (A === LB.SY && B === LB.HL) {
    return NO_BREAK;
  }
  // LB31: ÷
  return ALLOWED;
}

/** LB28a for the boundary between units a and b. */
function inAksara(units: Unit[], a: number, b: number, cls: (i: number) => number): boolean {
  const isDotted = (i: number): boolean => units[i]?.cp === DOTTED_CIRCLE;
  const aksara = (i: number): boolean => cls(i) === LB.AK || cls(i) === LB.AS || isDotted(i);
  const A = cls(a);
  const B = cls(b);
  return (A === LB.AP && aksara(b)) ||
    (aksara(a) && (B === LB.VF || B === LB.VI)) ||
    (A === LB.VI && aksara(a - 1) && (B === LB.AK || isDotted(b))) ||
    (aksara(a) && aksara(b) && cls(b + 1) === LB.VF);
}

function isAlpha(c: number): boolean {
  return c === LB.AL || c === LB.HL;
}

function isQuote(c: number): boolean {
  return c === LB.QU || c === LB.QI || c === LB.QF;
}

function isHangul(c: number): boolean {
  return [LB.JL, LB.JV, LB.JT, LB.H2, LB.H3].includes(c);
}
