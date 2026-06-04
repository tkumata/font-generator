#[must_use]
pub fn quantize_8_to_4(coverage: u8) -> u8 {
    let rounded = (u16::from(coverage) * 15 + 127) / 255;
    u8::try_from(rounded).unwrap_or(15)
}

#[must_use]
pub fn quantize_bitmap(coverage: &[u8]) -> Vec<u8> {
    coverage.iter().copied().map(quantize_8_to_4).collect()
}

#[must_use]
pub fn pack_4bit_alpha(alpha: &[u8]) -> Vec<u8> {
    alpha
        .chunks(2)
        .map(|chunk| {
            let high = chunk.first().copied().unwrap_or(0) & 0x0f;
            let low = chunk.get(1).copied().unwrap_or(0) & 0x0f;
            high << 4 | low
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ensure_eq<T>(actual: &T, expected: &T, field: &str) -> Result<(), Box<dyn std::error::Error>>
    where
        T: std::fmt::Debug + PartialEq,
    {
        if actual == expected {
            Ok(())
        } else {
            Err(std::io::Error::other(format!(
                "{field} mismatch: expected {expected:?}, got {actual:?}"
            ))
            .into())
        }
    }

    #[test]
    fn quantizes_coverage_with_rounding() -> Result<(), Box<dyn std::error::Error>> {
        ensure_eq(&quantize_8_to_4(0), &0, "transparent")?;
        ensure_eq(&quantize_8_to_4(255), &15, "opaque")?;
        ensure_eq(&quantize_8_to_4(128), &8, "half coverage")?;
        Ok(())
    }

    #[test]
    fn packs_two_4bit_alpha_values_per_byte() -> Result<(), Box<dyn std::error::Error>> {
        let packed = pack_4bit_alpha(&[0x0f, 0x03, 0x08, 0x00]);

        ensure_eq(&packed, &vec![0xf3, 0x80], "packed alpha")?;
        Ok(())
    }

    #[test]
    fn pads_final_low_nibble_for_odd_input() -> Result<(), Box<dyn std::error::Error>> {
        let packed = pack_4bit_alpha(&[0x01, 0x02, 0x03]);

        ensure_eq(&packed, &vec![0x12, 0x30], "odd packed alpha")?;
        Ok(())
    }
}
