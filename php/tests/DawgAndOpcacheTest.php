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
        $this->assertSame(25804, $dawg->getNumWords());
        $this->assertSame(18358, $dawg->getNumStates());

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

    public function testFromFileDoesNotRunPhpFiles(): void
    {
        $marker = sys_get_temp_dir() . '/thaibreak-ran-' . bin2hex(random_bytes(4));
        $file   = sys_get_temp_dir() . '/thaibreak-dict-' . bin2hex(random_bytes(4)) . '.php';
        file_put_contents($file, "<?php file_put_contents('$marker', 'ran'); return ['prefixes' => [], 'totalWeight' => 0.0];");
        try {
            try {
                DictionaryLoader::fromFile($file);
                $this->fail('fromFile() accepted a .php file');
            } catch (\InvalidArgumentException $e) {
                $this->assertStringContainsString('fromPhpFile', $e->getMessage());
            }
            $this->assertFileDoesNotExist($marker, 'the .php file was executed');
        } finally {
            @unlink($file);
            @unlink($marker);
        }
    }

    public function testWithDefaultDictRejectsMissingPaths(): void
    {
        $this->expectException(\InvalidArgumentException::class);
        ThaiTokenizer::withDefaultDict('/nonexistent/words.dawg');
    }

    public function testWithDefaultDictRejectsMissingBigramFile(): void
    {
        $this->expectException(\RuntimeException::class);
        ThaiTokenizer::withDefaultDict(null, '/nonexistent/bigrams.tsv');
    }

    public function testWithDefaultDictStillUsesTheBundledPhpDictionary(): void
    {
        $tok = ThaiTokenizer::withDefaultDict();
        $this->assertSame(['สวัสดี', 'ครับ'], $tok->tokenize('สวัสดีครับ'));
    }

    public function testCorruptDawgIsRejected(): void
    {
        $header = static fn (int $numStates, string $body = ''): string => 'TBD1' . pack('V', $numStates) . pack('V', 0) . $body;
        $cases = [
            'more states than bytes'   => $header(0xFFFFFFFF, "\0"),
            'no states'                => $header(0),
            'state runs past the end'  => $header(1, "\3a\0\0\0"),
            'truncated'                => $header(2, "\0"),
        ];
        foreach ($cases as $name => $data) {
            try {
                new CompactDawg($data);
                $this->fail("$name: CompactDawg accepted corrupt data");
            } catch (\InvalidArgumentException $e) {
                $this->assertStringContainsString('DAWG', $e->getMessage(), $name);
            }
        }
        $this->assertSame(1, (new CompactDawg($header(1, "\0")))->getNumStates());
    }
}
