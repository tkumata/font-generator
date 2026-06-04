#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "sample_font.h"

static const sample_font_glyph_t *find_glyph(
    const sample_font_font_size_t *font_size,
    const char *key
) {
    for (size_t index = 0; index < font_size->glyph_count; index += 1) {
        const sample_font_glyph_t *glyph = &font_size->glyphs[index];
        if (strcmp(glyph->key, key) == 0) {
            return glyph;
        }
    }
    return NULL;
}

int main(void) {
    if (sample_font_size_count == 0) {
        return 1;
    }

    const sample_font_font_size_t *font_size = &sample_font_sizes[0];
    const sample_font_glyph_t *glyph = find_glyph(font_size, "A");
    if (glyph == NULL) {
        return 1;
    }

    const uint8_t *bitmap = &font_size->bitmap_data[glyph->bitmap_offset];
    return bitmap[0] == 0xff ? 0 : 0;
}
