<?php

declare(strict_types=1);

namespace ThaiBreak\Tests;

use PHPUnit\Framework\TestCase;
use ThaiBreak\CompactDawg;
use ThaiBreak\DictionaryLoader;
use ThaiBreak\ThaiTokenizer;

class DawgAndOpcacheTest extends TestCase
{
    private string $dawgPath;
    private string $phpPath;
    private string $txtPath;

    protected function setUp(): void
    {
        $this->dawgPath = dirname(__DIR__, 2) . '/data/words.dawg';
        $this->phpPath  = dirname(__DIR__, 2) . '/data/words.php';
        $this->txtPath  = dirname(__DIR__, 2) . '/data/words.txt';
    }

    public function testCompactDawgBasic(): void
    {
        $dawg = CompactDawg::fromFile($this->dawgPath);
        $this->assertSame(25907, $dawg->getNumWords());
        $this->assertSame(17549, $dawg->getNumStates());

        $this->assertTrue($dawg->has('สวัสดี'));
        $this->assertTrue($dawg->has('กฎหมาย'));
        $this->assertFalse($dawg->has('คำไม่มีในดิกแน่นอน'));
    }

    public function testParityAcrossAllThreeBackends(): void
    {
        $trieDawg = DictionaryLoader::fromDawgFile($this->dawgPath);
        $triePhp  = DictionaryLoader::fromPhpFile($this->phpPath);
        $trieTxt  = DictionaryLoader::fromTextFile($this->txtPath);

        $tokDawg = new ThaiTokenizer($trieDawg);
        $tokPhp  = new ThaiTokenizer($triePhp);
        $tokTxt  = new ThaiTokenizer($trieTxt);

        $sentences = [
            'สถาบันวิจัยดาราศาสตร์แห่งชาติกระทรวงการอุดมศึกษาวิทยาศาสตร์วิจัยและนวัตกรรม',
            'กรุงเทพมหานครอมรรัตนโกสินทร์มหินทรายุธยามหาดิลกภพนพรัตน์ราชธานีบุรีรมย์',
            'ปัญญาประดิษฐ์กำลังเปลี่ยนแปลงวิถีชีวิตและระบบการทำงานของมนุษย์ทั่วโลก',
            'ทดสอบคำสั้นคำยาว กก กกหู กงการ กฎหมาย',
        ];

        foreach ($sentences as $s) {
            $wordsDawg = $tokDawg->tokenize($s, false);
            $wordsPhp  = $tokPhp->tokenize($s, false);
            $wordsTxt  = $tokTxt->tokenize($s, false);

            $this->assertSame($wordsTxt, $wordsDawg, "DAWG mismatch on: $s");
            $this->assertSame($wordsTxt, $wordsPhp, "PHP array mismatch on: $s");
        }
    }

    public function testDynamicAddOnDawg(): void
    {
        $trie = DictionaryLoader::fromDawgFile($this->dawgPath);
        $tok  = new ThaiTokenizer($trie);

        $sample = 'สวัสดีชาวโลกและทดสอบคำแปลกใหม่จ้า';
        $before = $tok->tokenize($sample, false);

        // Add dynamic word
        $trie->add('คำแปลกใหม่', 2.0);
        $after = $tok->tokenize($sample, false);

        $this->assertTrue($trie->has('คำแปลกใหม่'));
        $this->assertContains('คำแปลกใหม่', $after);
    }
}
