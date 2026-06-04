#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "sample_font.h"

static size_t utf8_unit_len(const char *text) {
    const unsigned char first = (unsigned char)text[0];
    if ((first & 0x80u) == 0u) {
        return 1;
    }
    if ((first & 0xE0u) == 0xC0u) {
        return 2;
    }
    if ((first & 0xF0u) == 0xE0u) {
        return 3;
    }
    if ((first & 0xF8u) == 0xF0u) {
        return 4;
    }
    return 0;
}

static int find_char_index(const char *key, size_t *out_index) {
    const char *cursor = sample_font_chars;
    for (size_t index = 0; index < SAMPLE_FONT_CHAR_COUNT; index += 1) {
        const size_t unit_len = utf8_unit_len(cursor);
        if (unit_len == 0u) {
            return 0;
        }
        if (strncmp(cursor, key, unit_len) == 0 && key[unit_len] == '\0') {
            *out_index = index;
            return 1;
        }
        cursor += unit_len;
    }
    return 0;
}

static uint8_t alpha_at(const uint8_t *bitmap, size_t pixel_index) {
    const uint8_t packed = bitmap[pixel_index / 2u];
    if ((pixel_index % 2u) == 0u) {
        return packed >> 4;
    }
    return packed & 0x0Fu;
}

int main(void) {
    size_t index = 0;
    if (!find_char_index("A", &index)) {
        return 1;
    }

    const uint8_t *bitmap = sample_font_data[index];
    const uint8_t top_left_alpha = alpha_at(bitmap, 0u);
    const uint8_t bottom_right_alpha =
        alpha_at(bitmap, (SAMPLE_FONT_WIDTH * SAMPLE_FONT_HEIGHT) - 1u);

    return (top_left_alpha <= 15u && bottom_right_alpha <= 15u) ? 0 : 1;
}
