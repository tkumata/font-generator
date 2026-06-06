# Specifications

## Command Shape

Proposed command:

```sh
font-generator --config fontgen.toml
```

Useful overrides:

```sh
font-generator --config fontgen.toml --size 16 --size 24 --language c
font-generator --font ./fonts/NotoSansJP-Regular.ttf --chars ./chars.txt --size 20 --language rust
```

## Configuration File

Recommended `fontgen.toml`:

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

## Character Files

Character files should contain the characters needed by firmware.

Example:

```text
abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ
!@#$%^&*()-_=+[]{}\;:'",<.>/?℃
☀️☁️☔️🌤️⛅️⛄️
あいうえおかきくけこ
湿度温度気圧設定空白削除漢字
```

The file does not need to be separated by alphabet, emoji, kana, or kanji. Grouping is allowed for readability, but generation should collect the final ordered unique display units from all configured files.

## Unicode Processing

The tool should treat visible user-facing units as display units rather than raw bytes.

Rules:

- UTF-8 input is required.
- Duplicate display units are removed.
- First occurrence determines output order.
- Newline characters are ignored by default.
- Space can be preserved when `preserve_space = true`.
- Character collection uses Unicode grapheme clusters.
- Emoji variation selectors and combined sequences remain associated with their base sequence when represented as one grapheme cluster.
- Empty input after whitespace filtering is an error.

Final MVP command behavior:

- The command prints normalized generation settings.
- The command reads configured character files.
- The command prints the collected display unit count and ordered display units.
- A preserved normal space is displayed as `<space>` in the summary.
- The command loads the configured TTF or OTF font.
- The command builds an in-memory rasterized font model for configured sizes.
- The command prints the generated model name, per-size glyph counts, per-size bitmap byte counts, and missing glyph report.
- The command writes generated output files for the configured language.
- The command prints the written output file paths.

## Font Sizes

Font size is a generation setting, not a character-list property.

Rules:

- Multiple sizes may be generated in one run.
- Sizes are configured in `fontgen.toml`.
- CLI `--size` values override configured sizes.
- Size values are positive integer pixel sizes.

## Alpha Format

Glyph bitmap pixels use 4-bit grayscale alpha.

Rules:

- Source rasterization coverage is normalized to 0 through 255.
- Stored alpha is quantized to 0 through 15.
- Two alpha pixels are packed into one byte.
- The high nibble stores the first pixel and the low nibble stores the second pixel.
- Odd pixel counts pad the final low nibble with 0.

Phase 4 stores the packed alpha bytes in memory. Phase 5 writes those bytes to C or Rust output files.

## Glyph Metadata

Metrics-based output records retain glyph metrics for advanced consumers.

Each glyph entry should include:

- Unicode display unit or lookup key.
- Bitmap offset.
- Packed bitmap length.
- Bitmap width.
- Bitmap height.
- Horizontal advance.
- X bearing.
- Y bearing or top offset.

Phase 4 model field names:

- `key`
- `bitmap_offset`
- `bitmap_len`
- `width`
- `height`
- `advance_x`
- `bearing_x`
- `bearing_y`

`fontdue` provides `xmin` and `ymin`; Phase 4 maps those to `bearing_x` and `bearing_y`.

## C Fixed Bitmap Output

The preferred C output targets microcontroller firmware that does not have a font renderer.

Configuration:

- `output.format = "c-fixed"` selects fixed-cell C output.
- `fixed_cell.width` sets the output cell width in pixels.
- `fixed_cell.height` sets the output cell height in pixels.
- `generation.sizes` must contain exactly one size for fixed-cell C output in Phase 8.

Generated C fixed output contains:

- A header guard.
- `#include <stdint.h>`.
- `{PREFIX}_WIDTH`.
- `{PREFIX}_HEIGHT`.
- `{PREFIX}_BPP`.
- `{PREFIX}_BYTES_PER_CHAR`.
- `{PREFIX}_CHAR_COUNT`.
- A UTF-8 character mapping string.
- A two-dimensional bitmap array with shape `[char_count][bytes_per_char]`.

For 4-bit alpha:

- `bytes_per_char = (width * height + 1) / 2`.
- High nibble stores the first pixel.
- Low nibble stores the second pixel.
- Odd pixel counts pad the final low nibble with 0.

Runtime contract:

- Firmware finds the display-unit index in the mapping.
- Firmware reads `bitmap_data[index]`.
- Firmware expands each nibble to alpha.
- Firmware draws the fixed cell at the caller-provided x and y position.
- Firmware does not read glyph metrics, advance values, bearings, or bitmap offsets.

README integration contract:

- Generate with `language = "c"` and `output.format = "c-fixed"`.
- Include the generated `{output_name}.h` in the firmware target.
- Treat `{output_name}_chars` as concatenated UTF-8 display units, not as a byte-indexed character array.
- Use the display-unit position in `{output_name}_chars` as the first index of `{output_name}_data`.
- Read pixels in row-major order over `{PREFIX}_WIDTH * {PREFIX}_HEIGHT`.
- Read an even pixel index from the high nibble and an odd pixel index from the low nibble.
- Interpret alpha as an integer from 0 through 15.
- Let firmware-specific code blend or otherwise map alpha to the target display format.

Glyph placement contract:

- Generation places each rasterized glyph into the configured fixed cell.
- The fixed cell origin is the only runtime placement origin.
- If a rasterized glyph does not fit the cell, generation must report a clear error unless a later explicit clipping option is added.
- Phase 8 does not add kerning, baseline layout, proportional text layout, or runtime shaping.

## Output Files

When `language = "c"` and the metrics format is selected:

- `app_font.h`
- `app_font.c`

When `language = "c"` and `output.format = "c-fixed"`:

- `app_font.h`

When `language = "rust"`:

- `app_font.rs`

Generated files are written to the current working directory by default.

Phase 5 C output includes:

- A header with glyph and font-size metadata structs.
- A source file with size-specific packed bitmap byte arrays.
- Size-specific glyph metadata arrays.
- A top-level `{name}_sizes` table.
- A top-level `{name}_size_count` value.

Phase 5 Rust output includes:

- `Glyph` and `FontSize` structs.
- Size-specific packed bitmap byte slices.
- Size-specific glyph metadata slices.
- A top-level `FONT_SIZES` slice.

The generated lookup data is intentionally table-based. Firmware can scan `glyphs` by `key` until Phase 6 examples define integration-specific helpers.

C fixed output replaces this recommendation for typical C firmware. New C firmware should prefer the fixed-cell format because it does not assume a font renderer.

## Examples

Repository examples:

- `examples/fontgen.toml`: sample generation config.
- `examples/chars.txt`: sample character list.
- `examples/c/minimal_usage.c`: C glyph lookup and bitmap slice example.
- `examples/rust/minimal_usage.rs`: Rust glyph lookup and bitmap slice example.

The examples assume the generated files use `sample_font` as the output name.

## Integration Documentation

`docs/INTEGRATION.md` documents:

- Common generated data contract.
- Packed 4-bit alpha format.
- ESP32 assumptions.
- Arduino assumptions.
- Raspberry Pi Pico W assumptions.
- Rust firmware assumptions.

## Operational Documentation

Final operational docs:

- `docs/SETUP.md`: supported hosts, Rust setup, macOS setup, Linux setup, and VS Code notes.
- `docs/CLI.md`: command options, command output, and exit behavior.
- `docs/CONFIGURATION.md`: TOML settings, path resolution, character rules, and rasterization limits.
- `docs/LIMITATIONS.md`: Unicode, shaping, font collection, lookup, rendering, memory placement, and licensing limits.
- `docs/TROUBLESHOOTING.md`: common configuration, font, glyph, output, and platform issues.

## Missing Glyph Policy

Supported policies:

- `error`: stop generation when a requested display unit cannot be rendered.
- `skip`: omit missing glyphs and report them.

The MVP should default to `error`.

Phase 4 reports two missing glyph reasons:

- `no font glyph`: the selected font does not map the scalar character to a glyph.
- `unsupported grapheme cluster`: the display unit contains multiple Unicode scalars and cannot be rasterized by the Phase 4 `fontdue` path.

## Determinism

For the same config, input files, font file, and tool version:

- Glyph order must be stable.
- Output byte arrays must be stable.
- Generated symbol names must be stable.
- Generated file ordering must be stable.
