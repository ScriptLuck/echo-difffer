use libpulse_binding::sample::Format;

// Volume calculation
pub fn calculate_volume(buffer: &[u8], format: &Format) -> f64 {
    match format {
        Format::U8 => {
            buffer
                .iter()
                .map(|&x| ((x as f64 - 128.0) / 128.0).powi(2))
                .sum::<f64>()
                / (buffer.len() as f64).sqrt()
        }

        Format::S16le => {
            buffer
                .chunks_exact(2)
                .map(|c| i16::from_le_bytes([c[0], c[1]]) as f64 / i16::MAX as f64)
                .map(|x| x.powi(2))
                .sum::<f64>()
                / ((buffer.len() / 2) as f64).sqrt()
        }

        Format::S32le => {
            buffer
                .chunks_exact(4) // 4 bytes per sample (32 bits)
                .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64 / i32::MAX as f64)
                .map(|x| x.powi(2))
                .sum::<f64>()
                / ((buffer.len() / 4) as f64).sqrt()
        }

        Format::F32le => {
            buffer
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64)
                .map(|x| x.powi(2))
                .sum::<f64>()
                / ((buffer.len() / 4) as f64).sqrt()
        }

        _ => panic!("Unsupported format: {:?}", format),
    }
}

// Volume adjustment
pub fn adjust_volume(buffer: &[u8], format: &Format, gain: f64) -> Vec<u8> {
    match format {
        Format::U8 => buffer
            .iter()
            .map(|&x| {
                ((x as f64 - 128.0) * gain + 128.0).clamp(u8::MIN as f64, u8::MAX as f64) as u8
            })
            .collect(),

        Format::S16le => buffer
            .chunks_exact(2)
            .flat_map(|c| {
                let sample = i16::from_le_bytes([c[0], c[1]]);
                let adjusted =
                    (sample as f64 * gain).clamp(i16::MIN as f64, i16::MAX as f64) as i16;
                adjusted.to_le_bytes()
            })
            .collect(),

        Format::S32le => buffer
            .chunks_exact(4)
            .flat_map(|c| {
                let sample = i32::from_le_bytes([c[0], c[1], c[2], c[3]]);
                let adjusted =
                    (sample as f64 * gain).clamp(i32::MIN as f64, i32::MAX as f64) as i32;
                adjusted.to_le_bytes()
            })
            .collect(),

        Format::F32le => buffer
            .chunks_exact(4)
            .flat_map(|c| {
                let sample = f32::from_le_bytes([c[0], c[1], c[2], c[3]]);
                let adjusted = (sample as f64 * gain).clamp(-1.0, 1.0) as f32;
                adjusted.to_le_bytes()
            })
            .collect(),

        _ => panic!("Unsupported format: {:?}", format),
    }
}
