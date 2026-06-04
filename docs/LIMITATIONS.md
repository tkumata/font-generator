# Known Limitations

## Unicode And Shaping

The MVP collects Unicode grapheme clusters, but rasterization currently supports only display units that map to one Unicode scalar.

Unsupported cases include:

- Emoji sequences with variation selectors.
- Multi-codepoint emoji.
- Ligature shaping.
- Complex script shaping that requires context.
- Color font layers.

These are reported as unsupported clusters or missing glyphs.

## Font Collections

The rasterizer currently uses the default face in a font collection. Some `.ttc` files may contain multiple faces, and the default face may not include every expected glyph.

## C Output Shape

The compatibility C output is metrics-based. It assumes the firmware can interpret glyph dimensions, bitmap offsets, advance, and bearings.

That assumption is too strong for many microcontroller projects. Use `output.format = "c-fixed"` for fixed-cell C bitmap output that can be drawn without a font renderer.

Use a font file or collection whose default face includes the requested characters.

## Generated Lookup Helpers

Generated files expose table data. They do not generate optimized lookup functions.

The examples show linear lookup by `key`. Firmware projects may replace that with a project-specific lookup strategy.

## Display Rendering

The generator does not emit display-driver code.

Firmware must provide:

- Pixel or span drawing.
- Color conversion.
- Alpha blending.
- Text layout policy.

## Memory Placement

Generated C arrays are `const`, but platform-specific placement such as `PROGMEM`, linker sections, or flash attributes is left to the firmware project.

## Font Licensing

The tool does not validate font licenses. Users must ensure the selected font can be used in their project.
