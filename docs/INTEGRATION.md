# Integration Notes

## Purpose

Phase 6 documents the assumptions needed to use generated font data in ESP32, Arduino, and Raspberry Pi Pico W projects.

The generator emits packed 4-bit alpha bitmap data and glyph metadata. It does not emit display-driver code.

## Common Data Contract

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

## ESP32

Assumptions:

- The firmware has a display driver that can draw individual pixels or spans.
- The generated C files are compiled into the firmware project.
- The renderer expands each nibble to a display-specific blend value.
- Font data can live in normal flash, or in a platform-specific const-data section added by the firmware project.

Integration steps:

- Generate C output.
- Add the generated `.h` and `.c` files to the ESP-IDF component or Arduino sketch.
- Include the generated header in the renderer.
- Scan the selected size table for `glyph->key`.
- Use `bitmap_offset`, `bitmap_len`, `width`, and `height` to read packed alpha data.

## Arduino

Assumptions:

- The target board has enough flash for the generated arrays.
- The display library accepts caller-provided pixel writes, spans, or monochrome/alpha masks.
- Any required `PROGMEM` placement is handled by the firmware project after generation.

Integration steps:

- Generate C output.
- Copy or include generated files in the sketch or library.
- Use the generated top-level size table to select a pixel size.
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
- Implement glyph lookup and nibble expansion in the display renderer.

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
