<?php

declare(strict_types=1);

namespace ThaiBreak;

/**
 * Unicode Line Breaking Algorithm (UAX #14) for Unicode 16.0.
 *
 * Classes are resolved as in LB1 (AI, SG, XX → AL; CJ → NS), except that
 * Complex_Context (SA) is kept as SA (letters) and SM (combining marks) so
 * Thai runs can be segmented with the dictionary. QU is split into QI
 * (\p{Pi}), QF (\p{Pf}) and plain QU for LB15a, LB15b and LB19.
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

    private const CLASS_MASK = 0x3F;
    private const FLAG_EAST_ASIAN = 0x40;
    private const FLAG_EXT_PICT_UNASSIGNED = 0x80;

    /** Packed table value (class id and flags) for a code point. */
    public static function value(int $cp): int
    {
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
        return LineBreakData::VALUES[$lo];
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
}
