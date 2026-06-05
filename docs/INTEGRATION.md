# Integration Notes

## Purpose

Phase 6 documents the assumptions needed to use generated font data in ESP32, Arduino, and Raspberry Pi Pico W projects.

The metrics format emits packed 4-bit alpha bitmap data and glyph metadata. It does not emit display-driver code.

The recommended C integration model is fixed-cell bitmap data for firmware that does not have a font renderer.

## Common Data Contract

### Metrics-Based MVP Contract

Generated glyph records include:

- `key`: UTF-8 display unit used for lookup.
- `bitmap_offset`: offset into the size-specific bitmap byte array.
- `bitmap_len`: number of packed bitmap bytes.
- `width`: bitmap width in pixels.
- `height`: bitmap height in pixels.
- `advance_x`: rounded horizontal advance in pixels.
- `bearing_x`: horizontal bitmap offset from the origin.
- `bearing_y`: vertical bitmap offset from the baseline.

Packed bitmap bytes store two 4-bit alpha pixels per byte:

- High nibble: first pixel.
- Low nibble: second pixel.
- Odd pixel counts pad the final low nibble with 0.

### Fixed-Cell C Contract

Fixed-cell C output is the preferred contract for ordinary microcontroller firmware.

Generated data includes:

- Fixed cell width.
- Fixed cell height.
- Bits per pixel.
- Bytes per character.
- Character count.
- UTF-8 character mapping.
- Bitmap data indexed by character mapping position.

Firmware usage:

- Find the display unit in the mapping.
- Select the bitmap record at the same index.
- Iterate `width * height` pixels.
- Expand each 4-bit alpha nibble.
- Draw to the display driver.

No glyph metrics, bearings, advance values, bitmap offsets, or variable glyph dimensions are required.

## ESP32

Assumptions:

- The firmware has a display driver that can draw individual pixels or spans.
- The generated C files are compiled into the firmware project.
- The firmware drawing loop expands each nibble to a display-specific blend value.
- Font data can live in normal flash, or in a platform-specific const-data section added by the firmware project.

Integration steps:

- Generate C output.
- Prefer fixed-cell C output for new firmware.
- Add the generated header to the ESP-IDF component or Arduino sketch.
- Use the character mapping index to select the fixed bitmap record.
- Expand nibbles while drawing the fixed cell.

## Arduino

Assumptions:

- The target board has enough flash for the generated arrays.
- The display library accepts caller-provided pixel writes, spans, or monochrome/alpha masks.
- Any required `PROGMEM` placement is handled by the firmware project after generation.

Integration steps:

- Generate C output.
- Copy or include generated files in the sketch or library.
- Prefer fixed-cell C output for new sketches.
- Use the character mapping index to select the fixed bitmap record.
- Expand each packed alpha byte into two alpha values before blending.

## Raspberry Pi Pico W

Assumptions:

- The generated files are compiled into the Pico SDK or Arduino-Pico project.
- The display path owns color conversion and alpha blending.
- The generator does not depend on USB, serial logging, or runtime host communication.

Integration steps:

- Generate C output for C or C++ firmware.
- Add generated files to the target build.
- Keep generated arrays in flash by leaving them `const`.
- Prefer fixed-cell C output for new firmware.
- Implement character index lookup and nibble expansion in the display drawing path.

## Rust Firmware

Assumptions:

- The target project can include a generated `.rs` module.
- The renderer can read `&'static [u8]` bitmap data.
- Heap allocation is not required.

Integration steps:

- Generate Rust output.
- Include the generated module in the firmware crate.
- Pick a `FontSize` from `FONT_SIZES`.
- Find the needed `Glyph` by matching `key`.
- Slice `bitmap_data` by `bitmap_offset` and `bitmap_len`.
