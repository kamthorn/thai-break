<?php

declare(strict_types=1);

namespace ThaiBreak\Laravel;

use Illuminate\Support\ServiceProvider;
use Illuminate\Support\Str;
use Illuminate\Support\Stringable;
use ThaiBreak\ThaiTokenizer;
use ThaiBreak\ThaiLineBreaker;

class ThaiBreakServiceProvider extends ServiceProvider
{
    /**
     * Register any application services.
     */
    public function register(): void
    {
        // Merge configuration
        $this->mergeConfigFrom(__DIR__ . '/../../config/thaibreak.php', 'thaibreak');

        // Bind ThaiTokenizer as a singleton
        $this->app->singleton(ThaiTokenizer::class, function ($app) {
            $config = $app['config']->get('thaibreak', []);
            $dictPath = $config['dict_path'] ?? null;
            $bigramPath = $config['bigram_path'] ?? null;

            $tokenizer = ThaiTokenizer::withDefaultDict($dictPath, $bigramPath);

            if (!empty($config['custom_words']) && is_array($config['custom_words'])) {
                $tokenizer->addCustomWords($config['custom_words']);
            }

            return $tokenizer;
        });

        $this->app->alias(ThaiTokenizer::class, 'thaibreak');
        $this->app->alias(ThaiTokenizer::class, 'thainlp'); // backward compatibility alias

        // Bind ThaiLineBreaker as a singleton
        $this->app->singleton(ThaiLineBreaker::class, function ($app) {
            return $app->make(ThaiTokenizer::class)->getLineBreaker();
        });

        $this->app->alias(ThaiLineBreaker::class, 'thaibreak.breaker');
        $this->app->alias(ThaiLineBreaker::class, 'thainlp.breaker');
    }

    /**
     * Bootstrap any package services.
     */
    public function boot(): void
    {
        // Publish config file
        if ($this->app->runningInConsole()) {
            $this->publishes([
                __DIR__ . '/../../config/thaibreak.php' => $this->app->configPath('thaibreak.php'),
            ], 'thaibreak-config');
        }

        // Register Blade directives
        $this->registerBladeDirectives();

        // Register Str and Stringable macros
        $this->registerStrMacros();

        // Eager preload dictionary if enabled (e.g. Octane / Swoole / Daemons)
        if ($this->app['config']->get('thaibreak.preload', false)) {
            $this->app->make(ThaiTokenizer::class);
        }
    }

    /**
     * Register Blade directives for Thai line breaking and wrapping.
     */
    protected function registerBladeDirectives(): void
    {
        if (!$this->app->bound('blade.compiler')) {
            return;
        }

        $blade = $this->app->make('blade.compiler');

        // @thaibreak($text, $isHtml = false) or @thailinebreak($text, $isHtml = false)
        $bladeDirectiveLineBreak = function ($expression) {
            return "<?php echo app('thaibreak')->insertLineBreaks({$expression}); ?>";
        };
        $blade->directive('thaibreak', $bladeDirectiveLineBreak);
        $blade->directive('thailinebreak', $bladeDirectiveLineBreak);

        // @thaiwrap($text, $width = 60)
        $blade->directive('thaiwrap', function ($expression) {
            return "<?php echo e(app('thaibreak')->wrap({$expression})); ?>";
        });
    }

    /**
     * Register Str and Stringable macros.
     */
    protected function registerStrMacros(): void
    {
        if (!class_exists(Str::class)) {
            return;
        }

        Str::macro('thaiWords', function (string $text, bool $keepWhitespace = false): array {
            return app('thaibreak')->tokenize($text, $keepWhitespace);
        });

        Str::macro('thaiTokenize', function (string $text, bool $keepWhitespace = false): array {
            return app('thaibreak')->tokenize($text, $keepWhitespace);
        });

        Str::macro('thaiLines', function (
            string $text,
            string $marker = ThaiLineBreaker::DEFAULT_MARKER,
            bool $isHtml = false
        ): string {
            return app('thaibreak')->insertLineBreaks($text, $marker, $isHtml);
        });

        Str::macro('thaiLineBreak', function (
            string $text,
            string $marker = ThaiLineBreaker::DEFAULT_MARKER,
            bool $isHtml = false
        ): string {
            return app('thaibreak')->insertLineBreaks($text, $marker, $isHtml);
        });

        Str::macro('thaiWrap', function (
            string $text,
            int $width = 60,
            string $break = "\n",
            bool $cutLongWords = false
        ): string {
            return app('thaibreak')->wrap($text, $width, $break, $cutLongWords);
        });

        if (class_exists(Stringable::class)) {
            Stringable::macro('thaiWords', function (bool $keepWhitespace = false): array {
                /** @var Stringable $this */
                return app('thaibreak')->tokenize((string) $this->value, $keepWhitespace);
            });

            Stringable::macro('thaiTokenize', function (bool $keepWhitespace = false): array {
                /** @var Stringable $this */
                return app('thaibreak')->tokenize((string) $this->value, $keepWhitespace);
            });

            Stringable::macro('thaiLines', function (
                string $marker = ThaiLineBreaker::DEFAULT_MARKER,
                bool $isHtml = false
            ): Stringable {
                /** @var Stringable $this */
                return new Stringable(
                    app('thaibreak')->insertLineBreaks((string) $this->value, $marker, $isHtml)
                );
            });

            Stringable::macro('thaiLineBreak', function (
                string $marker = ThaiLineBreaker::DEFAULT_MARKER,
                bool $isHtml = false
            ): Stringable {
                /** @var Stringable $this */
                return new Stringable(
                    app('thaibreak')->insertLineBreaks((string) $this->value, $marker, $isHtml)
                );
            });

            Stringable::macro('thaiWrap', function (
                int $width = 60,
                string $break = "\n",
                bool $cutLongWords = false
            ): Stringable {
                /** @var Stringable $this */
                return new Stringable(
                    app('thaibreak')->wrap((string) $this->value, $width, $break, $cutLongWords)
                );
            });
        }
    }

    /**
     * Get the services provided by the provider.
     *
     * @return array<int, string>
     */
    public function provides(): array
    {
        return [
            ThaiTokenizer::class,
            ThaiLineBreaker::class,
            'thaibreak',
            'thaibreak.breaker',
            'thainlp',
            'thainlp.breaker',
        ];
    }
}
