# Font Generator

Rust command line tool for generating compact microcontroller font data from project-specific character lists and TTF or OTF fonts.

Target firmware environments:

- ESP32
- Arduino
- Raspberry Pi Pico W

Host environments:

- macOS
- Linux

## Goal

Embedded display projects often need only a small subset of Japanese text, symbols, and emoji. Storing a complete font is usually too expensive for microcontrollers, so this project generates only the glyphs that firmware expects to use.

Generated glyph bitmaps use 4-bit grayscale alpha for pseudo-antialiased rendering.

The preferred C direction is fixed-cell bitmap output for firmware that does not have a font renderer. The generator should prepare data that a microcontroller can draw with direct character lookup, fixed-size bitmap records, and a small nibble-unpacking loop.

## Usage

```sh
cargo run -- --config fontgen.toml
cargo run -- --font ./fonts/NotoSansJP-Regular.ttf --chars ./chars.txt --size 20 --language rust
```

Example `fontgen.toml`:

```toml
[font]
path = "./fonts/NotoSansJP-Regular.ttf"

[input]
chars = ["./chars.txt"]
preserve_space = true

[output]
language = "c"
name = "app_font"
directory = "."
format = "c-fixed"

[generation]
sizes = [26]
alpha_bits = 4
missing_glyphs = "error"

[fixed_cell]
width = 26
height = 26
```

Useful overrides:

```sh
cargo run -- --config fontgen.toml --size 16 --size 24 --language c
```

The command prints normalized generation settings, reads configured character files, prints the ordered unique display units, loads the configured font, builds an in-memory rasterized font model, and writes output files for the configured language.

Supported options:

- `--config`: path to the TOML config file. When omitted, `fontgen.toml` is loaded if it exists; otherwise required settings must be provided through CLI arguments.
- `--font`: override `[font].path`.
- `--chars`: override `[input].chars`. May be passed multiple times.
- `--size`: override `[generation].sizes`. May be passed multiple times.
- `--language`: override `[output].language`. Supported values are `c` and `rust`.
- `--output-name`: override `[output].name`.
- `--output-dir`: override `[output].directory`.
- `--preserve-space`: override `[input].preserve_space` with `true` or `false`.

Example `chars.txt`:

```text
abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ
!@#$%^&*()-_=+[]{}\;:'",<.>/?℃
☀️☁️☔️🌤️⛅️⛄️
あいうえおかきくけこ
湿度温度気圧設定空白削除漢字
```

## Font Rasterization

The tool uses `fontdue` for TTF and OTF loading and grayscale rasterization.

Rasterization behavior:

- Single-scalar display units are looked up and rasterized through the selected font.
- Multi-scalar grapheme clusters, including variation-selector emoji clusters, are reported as unsupported clusters.
- `missing_glyphs = "error"` stops generation when a requested display unit cannot be rendered.
- `missing_glyphs = "skip"` omits unsupported or missing display units and reports them in the model summary.
- 8-bit coverage is quantized to 4-bit alpha with rounding.
- Two 4-bit alpha pixels are packed into each byte in the in-memory model.

## Output Generation

When `language = "c"` and metrics output is selected, the tool writes:

- `{output_name}.h`
- `{output_name}.c`

When `output.format = "c-fixed"`, C output is a fixed-cell header for rendererless firmware. Metrics-based C output remains available with `output.format = "c-metrics"` or by omitting `output.format`.

Fixed-cell C output writes:

- `{output_name}.h`

When `language = "rust"`, the tool writes:

- `{output_name}.rs`

Metrics output contains size-specific packed bitmap byte arrays, glyph metadata arrays, and a top-level size table. Fixed-cell C output contains constants, a UTF-8 mapping string, and fixed-size bitmap records. The command prints every written path.

## Using Generated C Fixed Font Data

The `c-fixed` format is header-only and is intended for firmware that can draw pixels but does not have a font renderer. The following examples assume:

```toml
[output]
language = "c"
name = "app_font"
directory = "./generated"
format = "c-fixed"
```

Generate the header:

```sh
mkdir -p generated
cargo run -- --config fontgen.toml
```

Add `generated/app_font.h` to the firmware include path and include it:

```c
#include "app_font.h"
```

The generated header provides:

- `APP_FONT_WIDTH` and `APP_FONT_HEIGHT`: generated full-width glyph dimensions in pixels.
- `APP_FONT_FULL_WIDTH`: generated full-width glyph width.
- `APP_FONT_HALF_WIDTH`: generated half-width ASCII glyph width.
- `APP_FONT_BPP`: bits per pixel, currently `4`.
- `APP_FONT_FULL_BYTES_PER_CHAR`: packed bytes in one full-width bitmap record.
- `APP_FONT_HALF_BYTES_PER_CHAR`: packed bytes in one half-width bitmap record.
- `APP_FONT_BYTES_PER_CHAR`: compatibility alias for full-width record bytes.
- `APP_FONT_CHAR_COUNT`: number of generated display units.
- `app_font_chars`: all generated display units concatenated as UTF-8.
- `app_font_widths[index]`: generated pixel width for display-unit position `index`.
- `app_font_data[index]`: the bitmap corresponding to display-unit position `index`.

`APP_FONT_CHAR_COUNT` is a display-unit count, not the byte length of `app_font_chars`. ASCII uses one UTF-8 byte, while Japanese characters normally use three. Parse UTF-8 units before comparing them.

The configured fixed-cell size is treated as the requested size. Generated bitmap records reserve one pixel of spacing: a configured 26 by 26 cell produces 25-pixel-high records. Non-ASCII display units are 25 pixels wide, while ASCII letters, digits, punctuation, symbols, and preserved spaces are 12 pixels wide.

The following platform-neutral example finds characters, unpacks alpha, blends RGB565 colors, and draws fixed-cell text through a firmware-provided pixel function:

```c
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "app_font.h"

typedef void (*put_pixel_fn)(int x, int y, uint16_t color);

static size_t utf8_unit_len(const char *text) {
    const uint8_t first = (uint8_t)text[0];
    if ((first & 0x80u) == 0u) return 1u;
    if ((first & 0xE0u) == 0xC0u) return 2u;
    if ((first & 0xF0u) == 0xE0u) return 3u;
    if ((first & 0xF8u) == 0xF0u) return 4u;
    return 0u;
}

static int find_glyph(const char *unit, size_t unit_len, size_t *out_index) {
    const char *cursor = app_font_chars;

    for (size_t index = 0; index < APP_FONT_CHAR_COUNT; ++index) {
        const size_t mapped_len = utf8_unit_len(cursor);
        if (mapped_len == 0u) return 0;
        if (mapped_len == unit_len && memcmp(cursor, unit, unit_len) == 0) {
            *out_index = index;
            return 1;
        }
        cursor += mapped_len;
    }
    return 0;
}

static uint8_t alpha_at(const uint8_t *bitmap, size_t pixel_index) {
    const uint8_t packed = bitmap[pixel_index / 2u];
    return (pixel_index % 2u) == 0u ? packed >> 4 : packed & 0x0Fu;
}

static uint16_t blend_rgb565(uint16_t fg, uint16_t bg, uint8_t alpha) {
    const uint32_t inverse = 15u - alpha;
    const uint32_t red = (((fg >> 11) & 0x1Fu) * alpha
                        + ((bg >> 11) & 0x1Fu) * inverse + 7u) / 15u;
    const uint32_t green = (((fg >> 5) & 0x3Fu) * alpha
                          + ((bg >> 5) & 0x3Fu) * inverse + 7u) / 15u;
    const uint32_t blue = ((fg & 0x1Fu) * alpha
                         + (bg & 0x1Fu) * inverse + 7u) / 15u;
    return (uint16_t)((red << 11) | (green << 5) | blue);
}

static void draw_glyph(
    size_t glyph_index,
    int origin_x,
    int origin_y,
    uint16_t foreground,
    uint16_t background,
    put_pixel_fn put_pixel
) {
    const uint8_t *bitmap = app_font_data[glyph_index];
    const size_t glyph_width = app_font_widths[glyph_index];

    for (size_t y = 0; y < APP_FONT_HEIGHT; ++y) {
        for (size_t x = 0; x < glyph_width; ++x) {
            const size_t pixel_index = y * glyph_width + x;
            const uint8_t alpha = alpha_at(bitmap, pixel_index);
            put_pixel(
                origin_x + (int)x,
                origin_y + (int)y,
                blend_rgb565(foreground, background, alpha)
            );
        }
    }
}

static void draw_text(
    const char *text,
    int x,
    int y,
    uint16_t foreground,
    uint16_t background,
    put_pixel_fn put_pixel
) {
    while (*text != '\0') {
        const size_t unit_len = utf8_unit_len(text);
        size_t glyph_index = 0u;
        if (unit_len == 0u) return;
        if (find_glyph(text, unit_len, &glyph_index)) {
            draw_glyph(glyph_index, x, y, foreground, background, put_pixel);
            x += (int)app_font_widths[glyph_index];
        } else {
            x += (int)APP_FONT_FULL_WIDTH;
        }
        text += unit_len;
    }
}
```

Call `draw_text("温度", x, y, foreground, background, lcd_put_pixel)` after adapting `lcd_put_pixel` to the target display driver. The example advances matched display units by `app_font_widths[index]`; newline handling, clipping, wrapping, transparent-background drawing, and missing-glyph fallback belong to the firmware.

Bitmap pixels are row-major. Each byte stores two 4-bit alpha values: the first pixel is in the high nibble and the second is in the low nibble. Alpha `0` selects the background and alpha `15` selects the foreground.

The current generator accepts only single Unicode scalar display units. Multi-scalar grapheme clusters such as variation-selector emoji are not present in `app_font_chars` unless a future rasterizer adds support.

## Examples

Sample inputs:

- `examples/fontgen.toml`
- `examples/chars.txt`

Minimal usage examples:

- `examples/c/minimal_usage.c`
- `examples/rust/minimal_usage.rs`

Integration notes:

- `docs/INTEGRATION.md`

## Operational Documentation

- `docs/SETUP.md`: macOS, Linux, and VS Code setup.
- `docs/CLI.md`: command options and exit behavior.
- `docs/CONFIGURATION.md`: TOML settings and path rules.
- `docs/LIMITATIONS.md`: MVP limits.
- `docs/TROUBLESHOOTING.md`: common failures and fixes.

## Current Capabilities

The tool can load configuration, apply CLI overrides, validate required files and settings, collect Unicode grapheme clusters from character files, remove duplicates in stable order, load a configured font, rasterize scalar glyphs for configured sizes, print an in-memory font model summary, and write generated output files.

Supported generated output:

- Fixed-cell C bitmap header for firmware without a font renderer.
- Metrics-based C header and source files for firmware that owns renderer logic.
- Rust source file with glyph metadata and packed bitmap bytes.

The fixed-cell C format is the recommended C output for ordinary microcontroller firmware. It exposes a UTF-8 mapping string and index-aligned bitmap records, so firmware only needs display-unit lookup, bitmap selection, nibble unpacking, and display drawing.

Known constraints:

- Only 4-bit grayscale alpha output is supported.
- Fixed-cell C output requires exactly one configured font size.
- Fixed-cell C output requires configured cell width and height.
- Multi-scalar grapheme clusters, including variation-selector emoji clusters, are reported as unsupported clusters.

## Development

Recommended VS Code extensions:

- `rust-lang.rust-analyzer`
- `tamasfe.even-better-toml`

Useful commands:

```sh
make check
make build
cargo run -- --help
```

`make check` runs formatting, Clippy with denied warnings, and tests. `make build` builds the crate with all features enabled.

The project currently uses a normal Cargo binary crate. No VS Code workspace-specific configuration is required.
