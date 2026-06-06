# Design

## Architecture

The project follows a clean separation between input, domain model, rasterization, and output generation.

```text
CLI
  -> Config Loader
  -> Character Collector
  -> Font Rasterizer
  -> Font Model
  -> Output Generator
```

## Modules

Planned Rust modules:

- `alpha`: 8-bit coverage to 4-bit alpha conversion and packing.
- `cli`: argument parsing and command dispatch.
- `config`: config file parsing and CLI override merging.
- `chars`: UTF-8 text loading and display unit collection.
- `font`: font loading and rasterization adapter.
- `model`: generated font domain structs.
- `output`: shared output interfaces.
- `output::c`: C output generation.
- `output::rust`: Rust output generation.
- `error`: application errors.

## Domain Model

Proposed core model:

```text
GeneratedFont
  name
  sizes

GeneratedFontSize
  pixel_size
  glyphs
  bitmap_data

Glyph
  key
  bitmap_offset
  bitmap_len
  width
  height
  advance_x
  bearing_x
  bearing_y
```

The model should avoid output-language-specific fields. C and Rust generators should translate the shared model into language-specific code.

Phase 8 adds an output-specific fixed-cell projection for C. The shared rasterized model may still contain metrics, but C fixed output consumes it at generation time and writes fixed-size bitmap records so firmware does not need metrics.

## Config Model

Config should be normalized before generation.

Normalization responsibilities:

- Resolve relative paths against the current working directory or config location according to the final implementation decision.
- Merge CLI overrides.
- Validate required fields.
- Deduplicate and sort only where sorting does not change user-visible glyph order.
- Keep character order based on input order.

## Character Segmentation

Character collection should use a Unicode-aware segmentation crate rather than byte or scalar-value iteration alone. This is important for emoji sequences and variation selectors.

Phase 3 implementation decision:

- Use `unicode-segmentation` grapheme clusters for character collection.
- Store display units as owned `String` values in a `CharacterSet`.
- Use a `HashSet` only for duplicate detection while preserving first-seen order in `Vec<String>`.
- Ignore newline, carriage return, and tab graphemes.
- Preserve normal spaces only when `preserve_space` is enabled.
- Return an explicit error when no display units remain after filtering.
- Validate whether the selected font rasterization library can render the collected cluster as intended.
- If the rasterizer only supports scalar glyphs, document the limitation and report unsupported clusters clearly.

## Rasterization

The rasterizer should be isolated behind a small interface so the rest of the project does not depend directly on a specific font crate.

Rasterization responsibilities:

- Load TTF or OTF font data.
- Resolve glyphs for requested display units.
- Rasterize each glyph at each requested size.
- Return bitmap coverage and metrics.

Phase 4 implementation decision:

- Use `fontdue` for font loading and rasterization.
- Load font bytes from `GenerationSettings::font_path`.
- Resolve only single-scalar display units through `Font::has_glyph`.
- Report multi-scalar grapheme clusters as unsupported instead of splitting them.
- Build `GeneratedFont` with one `GeneratedFontSize` per configured size.
- Append packed 4-bit alpha bytes into each size-specific `bitmap_data`.
- Store glyph bitmap offsets and lengths relative to that size-specific bitmap data.

## Alpha Packing

The alpha conversion is a pure transformation and should be heavily tested.

Algorithm:

1. Convert 8-bit coverage to 4-bit alpha using rounding.
2. Pack two 4-bit alpha values into one byte.
3. Pad the final low nibble with 0 when needed.

Phase 4 tests cover quantization endpoints, mid coverage rounding, even packing, and odd-length padding.

## Output Generation

C metrics output is suitable only for firmware that already has renderer-like code.

Existing C metrics structure:

- Header declares glyph metadata structs and extern font data.
- Source defines glyph tables, bitmap byte arrays, and font descriptors.
- Arrays should be `const`.
- Size-specific arrays are named from the validated output name and pixel size.
- Empty skipped-output arrays use a dummy element while the public count remains 0, so the generated C stays standard-compatible.

C fixed output is the preferred C format for ordinary microcontroller projects.

Expected C fixed structure:

- Header-only output by default.
- Compile-time constants for width, height, bits per pixel, bytes per character, and character count.
- One UTF-8 mapping string in input order.
- One `uint8_t` bitmap array with fixed `[char_count][bytes_per_char]` shape.
- No runtime glyph metrics.
- No runtime bitmap offsets.
- No runtime variable glyph dimensions.

Runtime C fixed usage:

1. Locate the display unit in the mapping.
2. Use the mapping index to select the fixed bitmap record.
3. Iterate the fixed cell pixels.
4. Expand each 4-bit alpha nibble.
5. Draw through the firmware display driver.

Rust output should be suitable for embedded Rust projects.

Expected Rust structure:

- Public constants for bitmap data and glyph tables.
- Structs or references that can be used without heap allocation.
- `#![allow(...)]` should be avoided unless there is a concrete generated-code reason.
- Phase 5 emits a plain Rust module with `Glyph`, `FontSize`, size-specific slices, and `FONT_SIZES`.

Phase 5 implementation decision:

- Add `output::write_output` as the language dispatch point.
- Keep file writing in output modules and keep model generation separate.
- Write C output as `{name}.h` and `{name}.c`.
- Write Rust output as `{name}.rs`.
- Return written paths to the CLI for summary output.
- Keep lookup helper functions deferred to examples or later integration work; Phase 5 provides deterministic lookup tables.

## Phase 8 Rendererless C Fixed Bitmap Design

Problem:

The existing C output assumes the target firmware has enough renderer logic to interpret `advance_x`, bearings, variable glyph dimensions, and bitmap offsets. That is not the common case for small microcontroller display firmware.

Decision:

- Add an explicit C fixed-cell output format.
- Keep the metrics-based output available only as a non-default advanced format.
- Generate data that can be used by a simple drawing loop.
- Perform glyph placement into the fixed cell at generation time.
- Treat a glyph that cannot fit the configured cell as a generation error in Phase 8.

Configuration design:

- Add an output format setting with at least `c-fixed` and `c-metrics`.
- Default C generation should prefer `c-fixed` after the migration decision is implemented.
- Add a fixed-cell settings group for width and height.
- Restrict Phase 8 C fixed output to one configured pixel size to avoid ambiguous macro names and runtime size selection.

Generation design:

- Rasterize the selected font size using the existing font path.
- Allocate a zero-filled fixed-cell alpha buffer for each glyph.
- Copy the rasterized glyph coverage into the fixed cell using deterministic placement rules.
- Pack the fixed-cell alpha buffer as 4-bit alpha.
- Emit every record with identical byte length.

Placement design:

- Phase 8 should use a simple, documented placement rule before adding optional tuning.
- The default rule is center the rasterized glyph bitmap in the fixed cell.
- If centering would still exceed the fixed cell, return an error with the display unit and required bitmap size.
- Baseline-sensitive placement can be added later only if a real firmware integration requires it.

Testing design:

- Unit-test bytes-per-character calculation.
- Unit-test nibble packing for fixed-cell buffers.
- Unit-test deterministic C fixed header rendering.
- Integration-test a small generated fixed C output file.
- Example-test direct bitmap lookup without glyph metrics.

## Phase 9 C Fixed Usage Documentation Design

The README usage section should follow the runtime sequence used by firmware:

- Generate one fixed-cell header.
- Add the header to the firmware build.
- Decode one UTF-8 display unit at a time.
- Find the matching display unit in the generated mapping.
- Select the bitmap at the same index.
- Unpack 4-bit alpha in row-major pixel order.
- Blend foreground and background colors in firmware-owned code.
- Advance by the generated fixed cell width.

The example should use small platform-neutral C helpers and a display callback so the generated data contract remains separate from ESP-IDF, Arduino, Pico SDK, or a particular LCD driver.

The README must distinguish byte length from character count. The mapping string is UTF-8, while `{PREFIX}_CHAR_COUNT` counts generated display units.

## Error Handling

Errors should be explicit and actionable.

Examples:

- Config file not found.
- Font file not found.
- Character file is not valid UTF-8.
- No characters found.
- No font sizes configured.
- Missing glyph for a requested display unit.
- Unsupported grapheme cluster for the selected rasterizer.
- Output path cannot be written.

## Testing Strategy

Unit tests:

- Config parsing.
- CLI override merge.
- Character deduplication.
- Unicode display unit handling.
- Alpha quantization.
- Nibble packing.
- Missing glyph and unsupported cluster reporting.
- Output symbol naming.
- C output rendering.
- Rust output rendering.
- Written output file reporting.

Integration tests:

- Sample config and character file produce deterministic output.
- C output and Rust output contain expected glyph records.
- Generated output files are written to the configured output directory.
- Minimal usage examples demonstrate table lookup and bitmap slicing.

## Phase 6 Examples

Example assets are kept under `examples/`.

The sample config uses a project-local font path placeholder. The repository does not vendor a font file, so users must provide a font under `fonts/` or override `--font`.

The C example includes the generated header and scans `sample_font_sizes[0].glyphs`.

The Rust example imports a generated `sample_font` module and scans `FONT_SIZES[0].glyphs`.

Integration assumptions are documented in `docs/INTEGRATION.md` instead of being embedded into generated code.

## Phase 7 Operational Documentation

Final documentation is split by operating concern:

- `docs/SETUP.md` covers host setup and VS Code notes.
- `docs/CLI.md` covers command-line behavior.
- `docs/CONFIGURATION.md` covers TOML configuration.
- `docs/LIMITATIONS.md` keeps MVP boundaries explicit.
- `docs/TROUBLESHOOTING.md` captures common operational failures.

This keeps README concise while preserving enough detail for a future agent or user to operate the MVP without reconstructing phase history.

## Phase Handoff Format

Each phase should update `docs/TASK/current-task.md` with:

- Current phase.
- Scope.
- Completed work.
- Remaining work.
- Commands run.
- Verification result.
- Next approval gate.

Before replacing `docs/TASK/current-task.md`, archive the previous task document using a `yyyymmddHHMM.md` filename.
