//! Presentation-only pressure colour and wind streaks. No simulation writes.
use crate::physics::Air;

/// The original's default air view: four-pixel pressure tiles, then red
/// streamlines. Work is bounded by the air grid and 84 raster samples per line,
/// regardless of how many charges are detonating. No allocations or SDL calls.
pub fn render_air(air: &Air, pixels: &mut [u32], width: usize, height: usize) {
    assert_eq!(pixels.len(), width * height);
    let size = air.cell_size;
    for cy in 0..air.rows {
        for cx in 0..air.cols {
            let node = air.nodes[cy * air.cols + cx];
            let intensity = (node.pressure.abs() * 48.).min(96.) as u32;
            let color = if node.pressure >= 0. {
                intensity << 8
            } else {
                intensity
            };
            let x = cx * size;
            let end = (x + size).min(width);
            for y in cy * size..((cy + 1) * size).min(height) {
                pixels[y * width + x..y * width + end].fill(color);
            }
        }
    }
    for cy in 0..air.rows {
        for cx in 0..air.cols {
            let node = air.nodes[cy * air.cols + cx];
            // Same inexpensive length estimate as the original's wind view.
            let ax = node.vx.abs();
            let ay = node.vy.abs();
            let speed = 0.9604 * ax.max(ay) + 0.3978 * ax.min(ay);
            if speed < 0.2 {
                continue;
            }
            let length = speed.min(8.);
            let scale = length * 10. / speed;
            let x = (cx * size + size / 2) as f32;
            let y = (cy * size + size / 2) as f32;
            let color = ((48. * length).min(96.) as u32) << 16;
            streak(
                pixels,
                width,
                height,
                (x + node.vx * scale, y + node.vy * scale),
                (x, y),
                color,
            );
        }
    }
}

fn streak(
    pixels: &mut [u32],
    width: usize,
    height: usize,
    from: (f32, f32),
    to: (f32, f32),
    color: u32,
) {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let steps = dx.abs().max(dy.abs()) as i32;
    let divisor = steps.max(1) as f32;
    let sx = (dx * 65536. / divisor).floor() as i32;
    let sy = (dy * 65536. / divisor).floor() as i32;
    let mut x = (from.0 * 65536.).floor() as i32 + 32768;
    let mut y = (from.1 * 65536.).floor() as i32 + 32768;
    for _ in 0..=steps {
        let px = x >> 16;
        let py = y >> 16;
        if px >= 0 && py >= 0 && (px as usize) < width && (py as usize) < height {
            pixels[py as usize * width + px as usize] = color;
        }
        x += sx;
        y += sy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressure_sign_and_intensity_match_the_original_palette() {
        let mut air = Air::new(9, 5);
        air.nodes[0].pressure = 1.;
        air.nodes[1].pressure = -1.;
        air.nodes[2].pressure = 20.;
        let mut pixels = vec![u32::MAX; 45];
        render_air(&air, &mut pixels, 9, 5);
        assert_eq!(pixels[0], 0x003000);
        assert_eq!(pixels[4], 0x000030);
        assert_eq!(pixels[8], 0x006000);
        assert_eq!(pixels[44], 0);
    }

    #[test]
    fn wind_streaks_follow_direction_and_stop_at_the_field_edge() {
        let mut air = Air::new(13, 9);
        air.nodes[0].vx = 12.;
        air.nodes[0].vy = -12.;
        air.nodes[2 * air.cols + 3].vx = -12.;
        let before = air.nodes.clone();
        let mut pixels = vec![0; 13 * 9];
        render_air(&air, &mut pixels, 13, 9);
        assert_eq!(air.nodes, before);
        assert_eq!(pixels[2 * 13 + 2], 0x600000);
        assert!(pixels.iter().any(|&p| p == 0x600000));
    }

    #[test]
    fn resting_air_erases_the_previous_wave_without_persistent_heat() {
        let air = Air::new(17, 11);
        let mut pixels = vec![0xffffff; 17 * 11];
        render_air(&air, &mut pixels, 17, 11);
        assert!(pixels.iter().all(|&p| p == 0));
    }
}
