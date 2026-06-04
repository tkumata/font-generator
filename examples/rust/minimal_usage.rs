mod sample_font;

fn find_glyph(font_size: &sample_font::FontSize, key: &str) -> Option<sample_font::Glyph> {
    font_size
        .glyphs
        .iter()
        .copied()
        .find(|glyph| glyph.key == key)
}

fn main() {
    let Some(font_size) = sample_font::FONT_SIZES.first() else {
        return;
    };

    let Some(glyph) = find_glyph(font_size, "A") else {
        return;
    };

    let bitmap_start = glyph.bitmap_offset;
    let bitmap_end = bitmap_start + glyph.bitmap_len;
    let _bitmap = &font_size.bitmap_data[bitmap_start..bitmap_end];
}
