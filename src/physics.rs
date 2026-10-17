//! Bounded, deterministic air and small rigid-body approximations for the sandbox.
use crate::sim::{Cell, Material};
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AirCell {
    pub vx: f32,
    pub vy: f32,
    pub pressure: f32,
}

pub struct Air {
    pub cols: usize,
    pub rows: usize,
    pub cell_size: usize,
    pub nodes: Vec<AirCell>,
    scratch: Vec<AirCell>,
    blocked: Vec<bool>,
}
impl Air {
    pub fn new(width: usize, height: usize) -> Self {
        let cell_size = 4;
        let cols = width.div_ceil(cell_size);
        let rows = height.div_ceil(cell_size);
        Self {
            cols,
            rows,
            cell_size,
            nodes: vec![AirCell::default(); cols * rows],
            scratch: vec![AirCell::default(); cols * rows],
            blocked: vec![false; cols * rows],
        }
    }
    pub fn clear(&mut self) {
        self.nodes.fill(AirCell::default());
        self.scratch.fill(AirCell::default());
    }
    fn inject(&mut self, x: f32, y: f32, vx: f32, vy: f32, p: f32, radius: f32) {
        if ![x, y, vx, vy, p, radius].iter().all(|v| v.is_finite()) {
            return;
        }
        let radius = radius.clamp(4., 160.);
        let s = self.cell_size as f32;
        let x0 = ((x - radius) / s).floor().max(0.) as usize;
        let y0 = ((y - radius) / s).floor().max(0.) as usize;
        let x1 = (((x + radius) / s).ceil().max(0.) as usize).min(self.cols.saturating_sub(1));
        let y1 = (((y + radius) / s).ceil().max(0.) as usize).min(self.rows.saturating_sub(1));
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                let distance = ((cx as f32 * s + s * 0.5 - x).powi(2)
                    + (cy as f32 * s + s * 0.5 - y).powi(2))
                .sqrt();
                let weight = (1. - distance / radius).max(0.);
                let n = &mut self.nodes[cy * self.cols + cx];
                n.vx = (n.vx + vx * weight).clamp(-12., 12.);
                n.vy = (n.vy + vy * weight).clamp(-12., 12.);
                n.pressure = (n.pressure + p * weight).clamp(-20., 20.);
            }
        }
    }
    pub fn add_wind(&mut self, x: f32, y: f32, vx: f32, vy: f32, radius: f32) {
        self.inject(x, y, vx, vy, 0., radius);
    }
    pub fn add_pressure(&mut self, x: f32, y: f32, amount: f32, radius: f32) {
        self.inject(x, y, 0., 0., amount, radius);
    }
    pub fn sample(&self, x: f32, y: f32) -> (f32, f32, f32) {
        if self.nodes.is_empty() || !x.is_finite() || !y.is_finite() {
            return (0., 0., 0.);
        }
        let cx = ((x.max(0.) / self.cell_size as f32) as usize).min(self.cols - 1);
        let cy = ((y.max(0.) / self.cell_size as f32) as usize).min(self.rows - 1);
        let n = self.nodes[cy * self.cols + cx];
        (n.vx, n.vy, n.pressure)
    }
    pub fn step(&mut self, width: usize, height: usize, cells: &[Cell]) {
        if cells.len() != width * height
            || self.cols != width.div_ceil(self.cell_size)
            || self.rows != height.div_ceil(self.cell_size)
        {
            return;
        }
        self.blocked.fill(false);
        // One solid pixel seals its coarse tile, so even a thin drawn wall is airtight.
        for (i, c) in cells.iter().enumerate() {
            if solid(c.material) {
                self.blocked
                    [(i / width / self.cell_size) * self.cols + (i % width / self.cell_size)] =
                    true;
            }
        }
        for y in 0..self.rows {
            for x in 0..self.cols {
                let i = y * self.cols + x;
                if self.blocked[i] {
                    self.scratch[i] = AirCell::default();
                    continue;
                }
                let c = self.nodes[i];
                let neighbor = |nx: isize, ny: isize| {
                    if nx < 0 || ny < 0 || nx >= self.cols as isize || ny >= self.rows as isize {
                        return AirCell {
                            pressure: c.pressure,
                            ..AirCell::default()
                        };
                    }
                    let j = ny as usize * self.cols + nx as usize;
                    if self.blocked[j] {
                        AirCell {
                            pressure: c.pressure,
                            ..AirCell::default()
                        }
                    } else {
                        self.nodes[j]
                    }
                };
                let l = neighbor(x as isize - 1, y as isize);
                let r = neighbor(x as isize + 1, y as isize);
                let u = neighbor(x as isize, y as isize - 1);
                let d = neighbor(x as isize, y as isize + 1);
                let mut vx = (c.vx * 0.78 + (l.vx + r.vx + u.vx + d.vx) * 0.05
                    - (r.pressure - l.pressure) * 0.19)
                    * 0.993;
                let mut vy = (c.vy * 0.78 + (l.vy + r.vy + u.vy + d.vy) * 0.05
                    - (d.pressure - u.pressure) * 0.19)
                    * 0.993;
                if x == 0 || x + 1 == self.cols || self.blocked[i - 1] || self.blocked[i + 1] {
                    vx *= 0.25;
                }
                if y == 0
                    || y + 1 == self.rows
                    || self.blocked[i - self.cols]
                    || self.blocked[i + self.cols]
                {
                    vy *= 0.25;
                }
                let pressure = (c.pressure * 0.8
                    + (l.pressure + r.pressure + u.pressure + d.pressure) * 0.05
                    - (r.vx - l.vx + d.vy - u.vy) * 0.16)
                    * 0.994;
                self.scratch[i] = AirCell {
                    vx: vx.clamp(-12., 12.),
                    vy: vy.clamp(-12., 12.),
                    pressure: pressure.clamp(-20., 20.),
                };
            }
        }
        std::mem::swap(&mut self.nodes, &mut self.scratch);
    }
}
fn solid(m: Material) -> bool {
    matches!(
        m,
        Material::Wall
            | Material::Metal
            | Material::Wood
            | Material::Ice
            | Material::Clone
            | Material::Glass
            | Material::Pump
            | Material::Fan
    )
}
fn at(cells: &[Cell], w: usize, h: usize, x: f32, y: f32) -> Material {
    if x < 0. || y < 0. || x >= w as f32 || y >= h as f32 {
        Material::Wall
    } else {
        cells[y as usize * w + x as usize].material
    }
}

pub const MAX_BODIES: usize = 128;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BodyKind {
    Wheel,
    Box,
    Bubble,
    Ball,
}
impl BodyKind {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Wheel),
            1 => Some(Self::Box),
            2 => Some(Self::Bubble),
            3 => Some(Self::Ball),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    pub kind: BodyKind,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub angle: f32,
    pub angular_velocity: f32,
    pub radius: f32,
    pub life: u16,
    pub payload: Material,
}
impl Body {
    pub fn new(kind: BodyKind, x: f32, y: f32, radius: f32) -> Self {
        Self {
            kind,
            x,
            y,
            vx: 0.,
            vy: 0.,
            angle: if kind == BodyKind::Box { 0.12 } else { 0. },
            angular_velocity: 0.,
            radius: radius.clamp(3., if kind == BodyKind::Bubble { 48. } else { 24. }),
            payload: if kind == BodyKind::Ball {
                Material::Sand
            } else {
                Material::Empty
            },
            life: if kind == BodyKind::Bubble {
                3600
            } else {
                u16::MAX
            },
        }
    }
    pub fn valid(&self, width: usize, height: usize) -> bool {
        [
            self.x,
            self.y,
            self.vx,
            self.vy,
            self.angle,
            self.angular_velocity,
            self.radius,
        ]
        .iter()
        .all(|v| v.is_finite())
            && self.x >= 0.
            && self.x < width as f32
            && self.y >= 0.
            && self.y < height as f32
            && self.vx.abs() <= 20.
            && self.vy.abs() <= 20.
            && self.angular_velocity.abs() <= 2.
            && (3. ..=if self.kind == BodyKind::Bubble {
                48.
            } else {
                24.
            })
                .contains(&self.radius)
            && self.life > 0
            && self.payload.is_cell()
    }
    fn point(&self, phase: f32) -> (f32, f32) {
        let (dx, dy) = if self.kind == BodyKind::Box {
            // Traverse an actual square perimeter, including its corners.
            let t = phase / TAU * 4.;
            let n = t.floor() as u8;
            let f = t.fract();
            match n {
                0 => (-1. + 2. * f, -1.),
                1 => (1., -1. + 2. * f),
                2 => (1. - 2. * f, 1.),
                _ => (-1., 1. - 2. * f),
            }
        } else {
            (phase.cos(), phase.sin())
        };
        let (s, c) = self.angle.sin_cos();
        (
            self.x + (dx * c - dy * s) * self.radius,
            self.y + (dx * s + dy * c) * self.radius,
        )
    }
}
fn collides(body: &Body, cells: &[Cell], w: usize, h: usize) -> Option<(f32, f32)> {
    let samples = (body.radius * 8.).ceil() as usize;
    for n in 0..samples {
        let (x, y) = body.point(n as f32 * TAU / samples as f32);
        if solid(at(cells, w, h, x, y)) {
            return Some((x - body.x, y - body.y));
        }
    }
    None
}

pub fn step_bodies(
    bodies: &mut Vec<Body>,
    cells: &mut [Cell],
    width: usize,
    height: usize,
    air: &mut Air,
) {
    bodies.truncate(MAX_BODIES);
    if cells.len() != width * height {
        return;
    }
    for b in bodies.iter_mut() {
        if !b.valid(width, height) {
            b.life = 0;
            continue;
        }
        let (wx, wy, p) = air.sample(b.x, b.y);
        let bubble = b.kind == BodyKind::Bubble;
        if bubble {
            b.life = b.life.saturating_sub(1);
            let dangerous = (0..32).any(|n| {
                let (x, y) = b.point(n as f32 * TAU / 32.);
                matches!(
                    at(cells, width, height, x, y),
                    Material::Fire
                        | Material::Magma
                        | Material::Spark
                        | Material::Acid
                        | Material::Thunder
                        | Material::Laser
                )
            });
            if dangerous || p.abs() > 5. {
                b.life = 0;
                air.add_pressure(b.x, b.y, 0.7, b.radius + 4.);
                continue;
            }
        }
        b.vx = (b.vx + (wx - b.vx) * if bubble { 0.08 } else { 0.012 }).clamp(-5., 5.);
        b.vy = (b.vy
            + if bubble {
                -0.025 + (wy - b.vy) * 0.08
            } else {
                0.12 + wy * 0.008
            })
        .clamp(-5., 5.);
        // The gear is fixed to its axle; only boxes and balls translate.
        if b.kind == BodyKind::Wheel {
            b.vx = 0.;
            b.vy = 0.;
        }
        b.angular_velocity = (b.angular_velocity + (wx * 0.6 + wy * 0.2) * 0.0015) * 0.99;
        // Tangential particle momentum drives the wheel and is returned through its spokes.
        if b.kind == BodyKind::Wheel {
            let mut torque = 0.;
            for n in 0..24 {
                let (x, y) = b.point(n as f32 * TAU / 24.);
                let m = at(cells, width, height, x, y);
                if m != Material::Empty && !solid(m) {
                    torque += (x - b.x) * 0.0008;
                }
            }
            b.angular_velocity = (b.angular_velocity + torque).clamp(-0.25, 0.25);
            for n in 0..4 {
                let a = b.angle + n as f32 * PI * 0.5;
                let dx = a.cos() * b.radius;
                let dy = a.sin() * b.radius;
                air.add_wind(
                    b.x + dx,
                    b.y + dy,
                    -dy * b.angular_velocity * 0.07,
                    dx * b.angular_velocity * 0.07,
                    5.,
                );
            }
        }
        let mut impact = false;
        let steps = b.vx.abs().max(b.vy.abs()).ceil().max(1.) as usize;
        for _ in 0..steps {
            let oldx = b.x;
            b.x += b.vx / steps as f32;
            if let Some((_, dy)) = collides(b, cells, width, height) {
                b.x = oldx;
                impact |= b.vx.abs() > 0.8;
                b.vx *= -if bubble { 0.3 } else { 0.55 };
                b.angular_velocity += dy * b.vx * 0.002;
            }
            let oldy = b.y;
            b.y += b.vy / steps as f32;
            if let Some((dx, _)) = collides(b, cells, width, height) {
                b.y = oldy;
                impact |= b.vy.abs() > 0.8;
                b.vy *= -if bubble {
                    0.3
                } else if b.kind == BodyKind::Ball {
                    0.88
                } else {
                    0.22
                };
                b.angular_velocity += dx * b.vy * 0.003;
                if b.kind == BodyKind::Wheel || b.kind == BodyKind::Ball {
                    b.angular_velocity += b.vx / b.radius * 0.08;
                }
            }
        }
        // Only an actual impact releases material: a resting ball cannot become
        // an unbounded source. Three checked slots preserve neighboring cells.
        if b.kind == BodyKind::Ball && impact && b.payload != Material::Empty {
            for offset in [-0.3_f32, 0., 0.3] {
                let a = -PI * 0.5 + offset;
                let x = (b.x + a.cos() * (b.radius + 2.)).round() as i32;
                let y = (b.y + a.sin() * (b.radius + 2.)).round() as i32;
                if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
                    let i = y as usize * width + x as usize;
                    if cells[i].material == Material::Empty {
                        cells[i] = Cell::new(b.payload);
                    }
                }
            }
        }
        let old = b.angle;
        b.angle = (b.angle + b.angular_velocity).rem_euclid(TAU);
        if collides(b, cells, width, height).is_some() {
            b.angle = old;
            b.angular_velocity *= -0.4;
        }
        b.angular_velocity = b.angular_velocity.clamp(-0.3, 0.3);
        // Filled bodies exclude loose particles throughout their interior. Project
        // displaced cells to the nearest free point just outside the outline.
        if matches!(b.kind, BodyKind::Box | BodyKind::Ball) {
            let extent = (b.radius * 1.42).ceil() as i32;
            let (s, c) = b.angle.sin_cos();
            for oy in -extent..=extent {
                for ox in -extent..=extent {
                    let x = b.x as i32 + ox;
                    let y = b.y as i32 + oy;
                    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
                        continue;
                    }
                    let dx = x as f32 - b.x;
                    let dy = y as f32 - b.y;
                    let distance = if b.kind == BodyKind::Box {
                        (dx * c + dy * s).abs().max((-dx * s + dy * c).abs())
                    } else {
                        (dx * dx + dy * dy).sqrt()
                    };
                    if distance >= b.radius {
                        continue;
                    }
                    let i = y as usize * width + x as usize;
                    if cells[i].material == Material::Empty || solid(cells[i].material) {
                        continue;
                    }
                    let (dx, dy, distance) = if distance < 0.1 {
                        (0., -1., 1.)
                    } else {
                        (dx, dy, distance)
                    };
                    for extra in 1..=4 {
                        let scale = (b.radius + extra as f32) / distance;
                        let nx = (b.x + dx * scale).round() as i32;
                        let ny = (b.y + dy * scale).round() as i32;
                        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                            break;
                        }
                        let j = ny as usize * width + nx as usize;
                        if solid(cells[j].material) {
                            break;
                        }
                        if cells[j].material == Material::Empty {
                            cells.swap(i, j);
                            break;
                        }
                    }
                }
            }
        }
        // Displace loose material into an empty neighbor instead of deleting it.
        // Bubbles only have a film; powder can fall through their hollow centers.
        for n in 0..32 {
            let (x, y) = b.point(n as f32 * TAU / 32.);
            if x < 1. || y < 1. || x >= width as f32 - 1. || y >= height as f32 - 1. {
                continue;
            }
            let i = y as usize * width + x as usize;
            let m = cells[i].material;
            if m == Material::Empty || solid(m) {
                continue;
            }
            let dx = (x - b.x).signum() as i32;
            let dy = (y - b.y).signum() as i32;
            let spin = if b.angular_velocity >= 0. { 1 } else { -1 };
            let directions = if b.kind == BodyKind::Wheel {
                [(-dy * spin, dx * spin), (dx, dy), (0, -1)]
            } else {
                [(dx, dy), (dx, 0), (0, dy)]
            };
            for (sx, sy) in directions {
                let nx = x as i32 + sx;
                let ny = y as i32 + sy;
                if nx >= 0 && ny >= 0 && nx < width as i32 && ny < height as i32 {
                    let j = ny as usize * width + nx as usize;
                    if cells[j].material == Material::Empty {
                        cells.swap(i, j);
                        break;
                    }
                }
            }
        }
    }
    // Pair collision work is bounded by MAX_BODIES; circles enclose square corners.
    for i in 0..bodies.len() {
        for j in i + 1..bodies.len() {
            let (left, right) = bodies.split_at_mut(j);
            let a = &mut left[i];
            let b = &mut right[0];
            if a.life == 0 || b.life == 0 {
                continue;
            }
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let dist = (dx * dx + dy * dy).sqrt();
            let bounding_radius = |body: &Body| {
                body.radius
                    * if body.kind == BodyKind::Box {
                        std::f32::consts::SQRT_2
                    } else {
                        1.
                    }
            };
            let limit = bounding_radius(a) + bounding_radius(b);
            if dist >= limit {
                continue;
            }
            if a.kind == BodyKind::Bubble || b.kind == BodyKind::Bubble {
                if a.kind == BodyKind::Bubble {
                    a.life = 0;
                }
                if b.kind == BodyKind::Bubble {
                    b.life = 0;
                }
                continue;
            }
            let (nx, ny) = if dist > 0.001 {
                (dx / dist, dy / dist)
            } else {
                (1., 0.)
            };
            let correction = (limit - dist) * 0.5;
            let mut proposed = *a;
            proposed.x -= nx * correction;
            proposed.y -= ny * correction;
            if a.kind != BodyKind::Wheel && collides(&proposed, cells, width, height).is_none() {
                a.x = proposed.x;
                a.y = proposed.y;
            }
            proposed = *b;
            proposed.x += nx * correction;
            proposed.y += ny * correction;
            if b.kind != BodyKind::Wheel && collides(&proposed, cells, width, height).is_none() {
                b.x = proposed.x;
                b.y = proposed.y;
            }
            let relative = (b.vx - a.vx) * nx + (b.vy - a.vy) * ny;
            if relative < 0. {
                let impulse = -relative * 0.8;
                a.vx -= impulse * nx;
                a.vy -= impulse * ny;
                b.vx += impulse * nx;
                b.vy += impulse * ny;
            }
        }
    }
    bodies.retain(|b| b.life > 0 && b.valid(width, height));
}

pub fn render_bodies(bodies: &[Body], pixels: &mut [u32], width: usize, height: usize) {
    if pixels.len() != width * height {
        return;
    }
    for b in bodies {
        let mut line = |x0: f32, y0: f32, x1: f32, y1: f32, color: u32| {
            let steps = ((x1 - x0).abs().max((y1 - y0).abs()).ceil() as usize).clamp(1, 160);
            for n in 0..=steps {
                let f = n as f32 / steps as f32;
                let x = (x0 + (x1 - x0) * f).round() as i32;
                let y = (y0 + (y1 - y0) * f).round() as i32;
                if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
                    pixels[y as usize * width + x as usize] = color;
                }
            }
        };
        let color = match b.kind {
            BodyKind::Wheel => 0xa9d7ff,
            BodyKind::Box => 0xd5b18a,
            BodyKind::Bubble => 0x89f5ff,
            BodyKind::Ball => {
                if b.payload == Material::Empty {
                    0xff8bd6
                } else {
                    b.payload.color()
                }
            }
        };
        for n in 0..48 {
            let a = n as f32 * TAU / 48.;
            let c = (n + 1) as f32 * TAU / 48.;
            let (mut x0, mut y0) = b.point(a);
            let (mut x1, mut y1) = b.point(c);
            if b.kind == BodyKind::Bubble {
                let deform = |t: f32| {
                    1. + 0.05 * (t * 3. + b.life as f32 * 0.05).sin() + b.vx.abs() * 0.025 * t.cos()
                };
                let d = deform(a);
                x0 = b.x + (x0 - b.x) * d;
                y0 = b.y + (y0 - b.y) / d;
                let d = deform(c);
                x1 = b.x + (x1 - b.x) * d;
                y1 = b.y + (y1 - b.y) / d;
            }
            line(
                x0,
                y0,
                x1,
                y1,
                if b.kind == BodyKind::Bubble && n % 12 < 3 {
                    0xffb9f5
                } else {
                    color
                },
            );
        }
        if b.kind == BodyKind::Wheel {
            for n in 0..6 {
                let (x, y) = b.point(n as f32 * TAU / 6.);
                line(b.x, b.y, x, y, color);
            }
        }
        if b.kind == BodyKind::Ball {
            let (x, y) = b.point(0.);
            line(b.x, b.y, x, y, 0xffe4f5);
        }
        if b.kind == BodyKind::Box {
            let (x, y) = b.point(0.);
            let (u, v) = b.point(PI);
            line(x, y, u, v, 0x86684c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pressure_creates_local_flow_and_wall_seals_it() {
        let mut cells = vec![Cell::default(); 64 * 64];
        let mut a = Air::new(64, 64);
        for y in 0..64 {
            cells[y * 64 + 32] = Cell::new(Material::Wall);
        }
        a.add_pressure(18., 30., 12., 10.);
        for _ in 0..35 {
            a.step(64, 64, &cells);
        }
        assert!(a.nodes.iter().any(|n| n.vx.abs() > 0.01));
        assert!(a
            .nodes
            .iter()
            .enumerate()
            .filter(|(i, _)| i % a.cols >= 9)
            .all(|(_, n)| n.vx == 0. && n.vy == 0. && n.pressure == 0.));
    }
    #[test]
    fn wind_propagates_and_decays() {
        let cells = vec![Cell::default(); 64 * 64];
        let mut a = Air::new(64, 64);
        a.add_wind(30., 30., 8., 0., 6.);
        let initial = a.sample(30., 30.).0;
        for _ in 0..8 {
            a.step(64, 64, &cells);
        }
        assert!(a.sample(38., 30.).0 > 0.);
        assert!(a.sample(30., 30.).0 < initial);
    }
    #[test]
    fn wind_rotates_wheel_and_box_falls_onto_thin_floor() {
        let mut cells = vec![Cell::default(); 64 * 64];
        for x in 0..64 {
            cells[50 * 64 + x] = Cell::new(Material::Wall);
        }
        let mut air = Air::new(64, 64);
        let mut bodies = vec![
            Body::new(BodyKind::Wheel, 18., 20., 6.),
            Body::new(BodyKind::Box, 42., 12., 5.),
        ];
        for _ in 0..80 {
            air.add_wind(18., 35., 2., 0., 30.);
            step_bodies(&mut bodies, &mut cells, 64, 64, &mut air);
        }
        assert_eq!(bodies.len(), 2);
        assert_eq!((bodies[0].x, bodies[0].y), (18., 20.));
        assert!(bodies[0].angle.abs() > 0.01);
        assert!(bodies[1].y > 35. && bodies[1].y < 46.);
        assert!((bodies[1].angle - 0.12).abs() > 0.01);
    }
    #[test]
    fn bubbles_float_then_pop_on_heat() {
        let mut cells = vec![Cell::default(); 64 * 64];
        let mut air = Air::new(64, 64);
        let mut bodies = vec![Body::new(BodyKind::Bubble, 30., 40., 5.)];
        for _ in 0..15 {
            step_bodies(&mut bodies, &mut cells, 64, 64, &mut air);
        }
        assert!(bodies[0].y < 39.);
        let (x, y) = bodies[0].point(0.);
        cells[y as usize * 64 + x as usize] = Cell::new(Material::Fire);
        step_bodies(&mut bodies, &mut cells, 64, 64, &mut air);
        assert!(bodies.is_empty());
    }
    #[test]
    fn ball_bounces_without_tunneling_and_particles_survive() {
        let mut cells = vec![Cell::default(); 64 * 64];
        for x in 0..64 {
            cells[50 * 64 + x] = Cell::new(Material::Wall);
        }
        cells[43 * 64 + 34] = Cell::new(Material::Sand);
        let mut air = Air::new(64, 64);
        let mut b = Body::new(BodyKind::Ball, 30., 35., 5.);
        b.vy = 5.;
        b.payload = Material::Empty;
        let mut bodies = vec![b];
        let mut bounced = false;
        for _ in 0..20 {
            step_bodies(&mut bodies, &mut cells, 64, 64, &mut air);
            assert!(bodies[0].y < 46.);
            bounced |= bodies[0].vy < -1.;
        }
        assert!(bounced);
        assert_eq!(
            cells
                .iter()
                .filter(|c| c.material == Material::Sand)
                .count(),
            1
        );
    }
    #[test]
    fn ball_releases_payload_on_impact_but_not_while_resting() {
        let mut cells = vec![Cell::default(); 64 * 64];
        for x in 0..64 {
            cells[50 * 64 + x] = Cell::new(Material::Wall);
        }
        let mut air = Air::new(64, 64);
        let mut ball = Body::new(BodyKind::Ball, 30., 44.8, 5.);
        ball.vy = 3.;
        ball.payload = Material::Water;
        let mut bodies = vec![ball];
        step_bodies(&mut bodies, &mut cells, 64, 64, &mut air);
        let count = cells
            .iter()
            .filter(|c| c.material == Material::Water)
            .count();
        assert!(count > 0 && count <= 3);
        bodies[0].y = 44.9;
        bodies[0].vy = 0.;
        for _ in 0..60 {
            step_bodies(&mut bodies, &mut cells, 64, 64, &mut air);
        }
        assert_eq!(
            cells
                .iter()
                .filter(|c| c.material == Material::Water)
                .count(),
            count
        );
    }
    #[test]
    fn dense_perimeter_detects_a_single_pixel_obstacle() {
        let mut cells = vec![Cell::default(); 96 * 96];
        cells[54 * 96 + 71] = Cell::new(Material::Wall);
        let body = Body::new(BodyKind::Ball, 48.5, 48.5, 24.);
        assert!(collides(&body, &cells, 96, 96).is_some());
    }
    #[test]
    fn pressure_pops_a_large_bubble() {
        let mut cells = vec![Cell::default(); 128 * 128];
        let mut air = Air::new(128, 128);
        let mut bodies = vec![Body::new(BodyKind::Bubble, 64., 64., 40.)];
        assert!(bodies[0].valid(128, 128));
        air.add_pressure(64., 64., 12., 24.);
        step_bodies(&mut bodies, &mut cells, 128, 128, &mut air);
        assert!(bodies.is_empty());
    }
    #[test]
    fn filled_box_expels_particles_without_erasing_them() {
        let mut cells = vec![Cell::default(); 64 * 64];
        cells[30 * 64 + 30] = Cell::new(Material::Sand);
        let mut bodies = vec![Body::new(BodyKind::Box, 30., 30., 8.)];
        let mut air = Air::new(64, 64);
        step_bodies(&mut bodies, &mut cells, 64, 64, &mut air);
        assert_eq!(
            cells
                .iter()
                .filter(|c| c.material == Material::Sand)
                .count(),
            1
        );
        assert_eq!(cells[30 * 64 + 30].material, Material::Empty);
    }
    #[test]
    fn body_count_and_air_memory_are_bounded() {
        let mut cells = vec![Cell::default(); 64 * 64];
        let mut air = Air::new(64, 64);
        let mut bodies = vec![Body::new(BodyKind::Ball, 30., 30., 4.); 1000];
        step_bodies(&mut bodies, &mut cells, 64, 64, &mut air);
        assert!(bodies.len() <= MAX_BODIES);
        let air = Air::new(384, 240);
        assert_eq!(air.nodes.len(), 5760);
        assert!(air.nodes.len() * std::mem::size_of::<AirCell>() * 2 < 150_000);
    }
}
