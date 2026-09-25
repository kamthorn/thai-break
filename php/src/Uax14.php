<?php

declare(strict_types=1);

namespace ThaiBreak;

/**
 * Unicode Line Breaking Algorithm (UAX #14) for Unicode 16.0.
 *
 * Classes are resolved as in LB1 (AI, SG, XX → AL; CJ → NS), except that
 * Complex_Context (SA) is kept as SA (letters) and SM (combining marks) so
 * Thai runs can be segmented with the dictionary. QU is split into QI
 * (\p{Pi}), QF (\p{Pf}) and plain QU for LB15a, LB15b and LB19. HH is not
 * assigned to any code point in Unicode 16.0 (U+2010 is BA, see LB20a).
 *
 * License: Apache-2.0
 */
final class Uax14
{
    public const AL = 0;
    public const BK = 1;
    public const CR = 2;
    public const LF = 3;
    public const NL = 4;
    public const SP = 5;
    public const ZW = 6;
    public const WJ = 7;
    public const GL = 8;
    public const CL = 9;
    public const CP = 10;
    public const EX = 11;
    public const IS = 12;
    public const SY = 13;
    public const OP = 14;
    public const QU = 15;
    public const QI = 16;
    public const QF = 17;
    public const NS = 18;
    public const B2 = 19;
    public const BA = 20;
    public const BB = 21;
    public const HY = 22;
    public const HH = 23;
    public const CB = 24;
    public const IN = 25;
    public const HL = 26;
    public const NU = 27;
    public const PR = 28;
    public const PO = 29;
    public const ID = 30;
    public const EB = 31;
    public const EM = 32;
    public const H2 = 33;
    public const H3 = 34;
    public const JL = 35;
    public const JV = 36;
    public const JT = 37;
    public const RI = 38;
    public const ZWJ = 39;
    public const CM = 40;
    public const AK = 41;
    public const AP = 42;
    public const AS = 43;
    public const VF = 44;
    public const VI = 45;
    public const SA = 46;
    public const SM = 47;

    /** Break actions returned by breakOpportunities() */
    public const NO_BREAK = 0;
    public const ALLOWED = 1;
    public const MANDATORY = 2;

    private const CLASS_MASK = 0x3F;
    private const FLAG_EAST_ASIAN = 0x40;
    private const FLAG_EXT_PICT_UNASSIGNED = 0x80;

    private const THAI_MAIYAMOK = 0x0E46;
    private const THAI_PAIYANNOI = 0x0E2F;
    private const HYPHEN = 0x2010;
    private const DOTTED_CIRCLE = 0x25CC;

    /** @var array<int, int> Memoized table values by code point */
    private static array $valueCache = [];

    /** Packed table value (class id and flags) for a code point. */
    public static function value(int $cp): int
    {
        if (isset(self::$valueCache[$cp])) {
            return self::$valueCache[$cp];
        }
        $starts = LineBreakData::STARTS;
        $lo = 0;
        $hi = count($starts) - 1;
        while ($lo < $hi) {
            $mid = ($lo + $hi + 1) >> 1;
            if ($starts[$mid] <= $cp) {
                $lo = $mid;
            } else {
                $hi = $mid - 1;
            }
        }
        return self::$valueCache[$cp] = LineBreakData::VALUES[$lo];
    }

    /** Resolved Line_Break class id of a code point (one of the class constants). */
    public static function lineBreakClass(int $cp): int
    {
        return self::value($cp) & self::CLASS_MASK;
    }

    /** Whether East_Asian_Width is F, W or H (the $EastAsian set of LB19a and LB30). */
    public static function isEastAsian(int $cp): bool
    {
        return (self::value($cp) & self::FLAG_EAST_ASIAN) !== 0;
    }

    /** Whether the code point is Extended_Pictographic and unassigned (LB30b). */
    public static function isExtPictUnassigned(int $cp): bool
    {
        return (self::value($cp) & self::FLAG_EXT_PICT_UNASSIGNED) !== 0;
    }

    /**
     * Compute the break action before every position of a code point sequence.
     *
     * Between two SA letters (Thai, Lao, Khmer, Myanmar) the dictionary decides:
     * a break is allowed where $dictBreaks[$i] is true. As a Thai tailoring,
     * there is never a break before ๆ (U+0E46) or ฯ (U+0E2F). When $dictBreaks
     * is null, SA is resolved to AL as in LB1 and no tailoring is applied.
     *
     * @param  list<int>       $cps        Code points
     * @param  list<bool>|null $dictBreaks Dictionary word boundaries indexed by position
     * @return list<int> Action before each position 0 … n (NO_BREAK, ALLOWED or MANDATORY);
     *                   position 0 is never a break (LB2), position n always is (LB3)
     */
    public static function breakOpportunities(array $cps, ?array $dictBreaks = null): array
    {
        $n = count($cps);
        $actions = array_fill(0, $n + 1, self::NO_BREAK);
        if ($n === 0) {
            return $actions;
        }
        $actions[$n] = self::MANDATORY;

        // LB9/LB10: group X (CM|ZWJ)* into units that take the class of X.
        $units = [];
        foreach ($cps as $i => $cp) {
            $value = self::value($cp);
            $cls   = $value & self::CLASS_MASK;
            if ($cls === self::SM) {
                $cls = self::CM;
            }
            if ($cls === self::CM || $cls === self::ZWJ) {
                $last = count($units) - 1;
                if ($last >= 0 && !in_array($units[$last]['cls'], [self::BK, self::CR, self::LF, self::NL, self::SP, self::ZW], true)) {
                    $units[$last]['zwj'] = $cls === self::ZWJ;
                    continue;
                }
            }
            $units[] = [
                'cls'   => match ($cls) {
                    self::CM, self::ZWJ => self::AL,
                    self::SA            => $dictBreaks === null ? self::AL : self::SA,
                    default             => $cls,
                },
                'cp'    => $cp,
                'start' => $i,
                'ea'    => ($value & self::FLAG_EAST_ASIAN) !== 0,
                'xp'    => ($value & self::FLAG_EXT_PICT_UNASSIGNED) !== 0,
                'zwj'   => $cls === self::ZWJ,
            ];
        }

        $count = count($units);
        for ($u = 1; $u < $count; $u++) {
            $actions[$units[$u]['start']] = self::pairAction($units, $u, $dictBreaks);
        }

        return $actions;
    }

    /**
     * Apply rules LB4–LB31 in order to the boundary before unit $b.
     *
     * @param list<array{cls:int,cp:int,start:int,ea:bool,xp:bool,zwj:bool}> $units
     * @param list<bool>|null $dictBreaks
     */
    private static function pairAction(array $units, int $b, ?array $dictBreaks): int
    {
        $a     = $b - 1;
        $count = count($units);
        $A     = $units[$a]['cls'];
        $B     = $units[$b]['cls'];
        $cls   = static fn(int $i): ?int => ($i >= 0 && $i < $count) ? ($units[$i]['cls'] === self::SA ? self::AL : $units[$i]['cls']) : null;

        // LB4–LB6: mandatory breaks
        if ($A === self::BK) {
            return self::MANDATORY;
        }
        if ($A === self::CR && $B === self::LF) {
            return self::NO_BREAK;
        }
        if ($A === self::CR || $A === self::LF || $A === self::NL) {
            return self::MANDATORY;
        }
        if (in_array($B, [self::BK, self::CR, self::LF, self::NL], true)) {
            return self::NO_BREAK;
        }
        // LB7: × SP, × ZW
        if ($B === self::SP || $B === self::ZW) {
            return self::NO_BREAK;
        }
        // Last unit before the boundary, skipping spaces (for the "X SP*" rules)
        $k = $a;
        while ($k > 0 && $units[$k]['cls'] === self::SP) {
            $k--;
        }
        $K = $cls($k);
        // LB8: ZW SP* ÷
        if ($K === self::ZW) {
            return self::ALLOWED;
        }
        // LB8a: ZWJ ×
        if ($units[$a]['zwj']) {
            return self::NO_BREAK;
        }
        // SA resolution (LB1), Thai tailoring: ๆ and ฯ never start a line, even after spaces
        if ($dictBreaks !== null && ($units[$b]['cp'] === self::THAI_MAIYAMOK || $units[$b]['cp'] === self::THAI_PAIYANNOI)) {
            return self::NO_BREAK;
        }
        // SA resolution (LB1): the dictionary decides inside SA runs
        if ($A === self::SA && $B === self::SA) {
            return ($dictBreaks[$units[$b]['start']] ?? false) ? self::ALLOWED : self::NO_BREAK;
        }
        $A = $cls($a);
        $B = $cls($b);
        // LB13: × CL, × CP, × EX, × SY
        if (in_array($B, [self::CL, self::CP, self::EX, self::SY], true)) {
            return self::NO_BREAK;
        }
        // LB15c: SP ÷ IS NU
        if ($A === self::SP && $B === self::IS && $cls($b + 1) === self::NU) {
            return self::ALLOWED;
        }
        // LB15d: × IS
        if ($B === self::IS) {
            return self::NO_BREAK;
        }
        // LB18: SP ÷
        if ($A === self::SP) {
            return self::ALLOWED;
        }
        // LB20a: (sot | BK | CR | LF | NL | SP | ZW | CB | GL) (HY | [\u2010]) × AL
        if (($A === self::HY || $units[$a]['cp'] === self::HYPHEN) && $B === self::AL
            && ($a === 0 || in_array($cls($a - 1), [self::BK, self::CR, self::LF, self::NL, self::SP, self::ZW, self::CB, self::GL], true))) {
            return self::NO_BREAK;
        }
        // LB21: × BA, × HY, × NS, BB ×
        if (in_array($B, [self::BA, self::HY, self::NS], true) || $A === self::BB) {
            return self::NO_BREAK;
        }
        // LB21a: HL (HY | [BA - $EastAsian]) × [^HL]
        if (($A === self::HY || ($A === self::BA && !$units[$a]['ea'])) && $cls($a - 1) === self::HL && $B !== self::HL) {
            return self::NO_BREAK;
        }
        // LB21b: SY × HL
        if ($A === self::SY && $B === self::HL) {
            return self::NO_BREAK;
        }
        // LB24: (PR | PO) × (AL | HL), (AL | HL) × (PR | PO)
        if ((($A === self::PR || $A === self::PO) && self::isAlpha($B))
            || (self::isAlpha($A) && ($B === self::PR || $B === self::PO))) {
            return self::NO_BREAK;
        }
        // LB25: numbers
        if ($B === self::PO || $B === self::PR) {
            // NU (SY | IS)* (CL | CP)? × (PO | PR)
            $j = ($A === self::CL || $A === self::CP) ? $a - 1 : $a;
            while (in_array($cls($j), [self::SY, self::IS], true)) {
                $j--;
            }
            if ($cls($j) === self::NU) {
                return self::NO_BREAK;
            }
        }
        if (($A === self::PO || $A === self::PR) && $B === self::OP
            && ($cls($b + 1) === self::NU || ($cls($b + 1) === self::IS && $cls($b + 2) === self::NU))) {
            return self::NO_BREAK; // (PO | PR) × OP IS? NU
        }
        if ($B === self::NU && in_array($A, [self::PO, self::PR, self::HY, self::IS], true)) {
            return self::NO_BREAK; // (PO | PR | HY | IS) × NU
        }
        if ($B === self::NU) {
            // NU (SY | IS)* × NU
            $j = $a;
            while (in_array($cls($j), [self::SY, self::IS], true)) {
                $j--;
            }
            if ($cls($j) === self::NU) {
                return self::NO_BREAK;
            }
        }
        // LB27: (JL | JV | JT | H2 | H3) × PO, PR × (JL | JV | JT | H2 | H3)
        if ((self::isHangul($A) && $B === self::PO) || ($A === self::PR && self::isHangul($B))) {
            return self::NO_BREAK;
        }
        // LB29: IS × (AL | HL)
        if ($A === self::IS && self::isAlpha($B)) {
            return self::NO_BREAK;
        }
        // LB31: ÷
        return self::ALLOWED;
    }

    /**
     * LB28a for the boundary between units $a and $b.
     *
     * @param list<array{cls:int,cp:int,start:int,ea:bool,xp:bool,zwj:bool}> $units
     * @param callable(int): ?int $cls
     */
    private static function inAksara(array $units, int $a, int $b, callable $cls): bool
    {
        $aksara = static fn(int $i): bool => in_array($cls($i), [self::AK, self::AS], true)
            || (($units[$i]['cp'] ?? null) === self::DOTTED_CIRCLE);
        $A = $cls($a);
        $B = $cls($b);
        return ($A === self::AP && $aksara($b))
            || ($aksara($a) && ($B === self::VF || $B === self::VI))
            || ($A === self::VI && $aksara($a - 1) && ($B === self::AK || $units[$b]['cp'] === self::DOTTED_CIRCLE))
            || ($aksara($a) && $aksara($b) && $cls($b + 1) === self::VF);
    }

    private static function isAlpha(?int $cls): bool
    {
        return $cls === self::AL || $cls === self::HL;
    }

    private static function isQuote(?int $cls): bool
    {
        return $cls === self::QU || $cls === self::QI || $cls === self::QF;
    }

    private static function isHangul(?int $cls): bool
    {
        return in_array($cls, [self::JL, self::JV, self::JT, self::H2, self::H3], true);
    }
}
