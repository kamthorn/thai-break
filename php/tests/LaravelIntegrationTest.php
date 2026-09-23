<?php

declare(strict_types=1);

namespace ThaiBreak\Tests;

use Illuminate\Support\Facades\Blade;
use Illuminate\Support\Str;
use Orchestra\Testbench\TestCase;
use ThaiBreak\Laravel\Facades\ThaiBreak;
use ThaiBreak\Laravel\ThaiBreakServiceProvider;
use ThaiBreak\ThaiLineBreaker;
use ThaiBreak\ThaiTokenizer;

class LaravelIntegrationTest extends TestCase
{
    protected function getPackageProviders($app): array
    {
        return [
            ThaiBreakServiceProvider::class,
        ];
    }

    protected function getPackageAliases($app): array
    {
        return [
            'ThaiBreak' => ThaiBreak::class,
        ];
    }

    public function testServiceContainerBindings(): void
    {
        $this->assertTrue($this->app->bound(ThaiTokenizer::class));
        $this->assertTrue($this->app->bound('thaibreak'));
        $this->assertTrue($this->app->bound('thainlp')); // backward compatibility
        $this->assertTrue($this->app->bound(ThaiLineBreaker::class));
        $this->assertTrue($this->app->bound('thaibreak.breaker'));

        $tokenizer = $this->app->make(ThaiTokenizer::class);
        $this->assertInstanceOf(ThaiTokenizer::class, $tokenizer);

        $aliasTokenizer = $this->app->make('thaibreak');
        $this->assertSame($tokenizer, $aliasTokenizer);

        $breaker = $this->app->make('thaibreak.breaker');
        $this->assertInstanceOf(ThaiLineBreaker::class, $breaker);
    }

    public function testFacade(): void
    {
        $words = ThaiBreak::words('ฉันรักภาษาไทย');
        $this->assertSame(['ฉัน', 'รัก', 'ภาษา', 'ไทย'], $words);

        $words2 = ThaiBreak::tokenize('ฉันรักภาษาไทย');
        $this->assertSame(['ฉัน', 'รัก', 'ภาษา', 'ไทย'], $words2);

        $joined = ThaiBreak::join('สวัสดีครับ', '|');
        $this->assertSame('สวัสดี|ครับ', $joined);

        $broken = ThaiBreak::lines('ฉันรักภาษาไทย');
        $this->assertStringContainsString(ThaiLineBreaker::DEFAULT_MARKER, $broken);

        $wrapped = ThaiBreak::wrap('ประเทศไทยมีวัฒนธรรมที่สวยงามและหลากหลาย', 20);
        $this->assertStringContainsString("\n", $wrapped);
    }

    public function testStrMacros(): void
    {
        $words = Str::thaiWords('ฉันรักภาษาไทย');
        $this->assertSame(['ฉัน', 'รัก', 'ภาษา', 'ไทย'], $words);

        $words2 = Str::thaiTokenize('ฉันรักภาษาไทย');
        $this->assertSame(['ฉัน', 'รัก', 'ภาษา', 'ไทย'], $words2);

        $broken = Str::thaiLines('ฉันรักภาษาไทย');
        $this->assertStringContainsString(ThaiLineBreaker::DEFAULT_MARKER, $broken);

        $broken2 = Str::thaiLineBreak('ฉันรักภาษาไทย');
        $this->assertStringContainsString(ThaiLineBreaker::DEFAULT_MARKER, $broken2);

        $wrapped = Str::thaiWrap('ประเทศไทยมีวัฒนธรรมที่สวยงามและหลากหลาย', 20);
        $this->assertStringContainsString("\n", $wrapped);
    }

    public function testStringableMacros(): void
    {
        $words = Str::of('ฉันรักภาษาไทย')->thaiWords();
        $this->assertSame(['ฉัน', 'รัก', 'ภาษา', 'ไทย'], $words);

        $words2 = Str::of('ฉันรักภาษาไทย')->thaiTokenize();
        $this->assertSame(['ฉัน', 'รัก', 'ภาษา', 'ไทย'], $words2);

        $broken = Str::of('ฉันรักภาษาไทย')->thaiLines();
        $this->assertInstanceOf(\Illuminate\Support\Stringable::class, $broken);
        $this->assertStringContainsString(ThaiLineBreaker::DEFAULT_MARKER, (string) $broken);

        $broken2 = Str::of('ฉันรักภาษาไทย')->thaiLineBreak();
        $this->assertInstanceOf(\Illuminate\Support\Stringable::class, $broken2);
        $this->assertStringContainsString(ThaiLineBreaker::DEFAULT_MARKER, (string) $broken2);

        $wrapped = Str::of('ประเทศไทยมีวัฒนธรรมที่สวยงามและหลากหลาย')->thaiWrap(20);
        $this->assertInstanceOf(\Illuminate\Support\Stringable::class, $wrapped);
        $this->assertStringContainsString("\n", (string) $wrapped);
    }

    public function testBladeDirectives(): void
    {
        $blade = $this->app->make('blade.compiler');

        $compiledThaiBreak = $blade->compileString("@thaibreak('สวัสดีครับ', false)");
        $this->assertStringContainsString("app('thaibreak')->insertLineBreaks('สวัสดีครับ', false)", $compiledThaiBreak);

        $compiledLineBreak = $blade->compileString("@thailinebreak('สวัสดีครับ', false)");
        $this->assertStringContainsString("app('thaibreak')->insertLineBreaks('สวัสดีครับ', false)", $compiledLineBreak);

        $compiledWrap = $blade->compileString("@thaiwrap('สวัสดีครับ', 40)");
        $this->assertStringContainsString("app('thaibreak')->wrap('สวัสดีครับ', 40)", $compiledWrap);
    }
}
