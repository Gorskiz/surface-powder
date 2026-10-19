//! Original compact cellular sandbox. No upstream game code or assets are used.

use crate::physics::{clean_residue, render_bodies, step_bodies, Air, Body, BodyKind, MAX_BODIES};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Material {
    #[default]
    Empty,
    Sand,
    Water,
    Fire,
    Seed,
    Wood,
    Gunpowder,
    Oil,
    Stone,
    Magma,
    Ice,
    Steam,
    Acid,
    Metal,
    Spark,
    Gas,
    Clone,
    Wall,
    Player,
    Fighter,
    Fan,
    Superball,
    Firework,
    C4,
    Virus,
    Nitro,
    Ant,
    Torch,
    Soapy,
    Thunder,
    Bomb,
    Laser,
    Vine,
    Salt,
    SaltWater,
    Glass,
    Bird,
    Mercury,
    Fuse,
    Cloud,
    Pump,
    Snow,
    Wind,
    Air,
    Drag,
    Wheel,
    Box,
    Bubble,
    Ball,
    Create,
}

impl Material {
    pub const ALL: [Self; 50] = [
        Self::Empty,
        Self::Sand,
        Self::Water,
        Self::Fire,
        Self::Seed,
        Self::Wood,
        Self::Gunpowder,
        Self::Oil,
        Self::Stone,
        Self::Magma,
        Self::Ice,
        Self::Steam,
        Self::Acid,
        Self::Metal,
        Self::Spark,
        Self::Gas,
        Self::Clone,
        Self::Wall,
        Self::Player,
        Self::Fighter,
        Self::Fan,
        Self::Superball,
        Self::Firework,
        Self::C4,
        Self::Virus,
        Self::Nitro,
        Self::Ant,
        Self::Torch,
        Self::Soapy,
        Self::Thunder,
        Self::Bomb,
        Self::Laser,
        Self::Vine,
        Self::Salt,
        Self::SaltWater,
        Self::Glass,
        Self::Bird,
        Self::Mercury,
        Self::Fuse,
        Self::Cloud,
        Self::Pump,
        Self::Snow,
        Self::Wind,
        Self::Air,
        Self::Drag,
        Self::Wheel,
        Self::Box,
        Self::Bubble,
        Self::Ball,
        Self::Create,
    ];
    pub fn from_u8(value: u8) -> Option<Self> {
        Self::ALL.get(value as usize).copied()
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Empty => "ERASE",
            Self::Sand => "POWDER",
            Self::Water => "WATER",
            Self::Fire => "FIRE",
            Self::Seed => "SEED",
            Self::Wood => "WOOD",
            Self::Gunpowder => "GUNPOWDER",
            Self::Oil => "OIL",
            Self::Stone => "STONE",
            Self::Magma => "MAGMA",
            Self::Ice => "ICE",
            Self::Steam => "STEAM",
            Self::Acid => "ACID",
            Self::Metal => "METAL",
            Self::Spark => "SPARK",
            Self::Gas => "GAS",
            Self::Clone => "CLONE",
            Self::Wall => "WALL",
            Self::Player => "PLAYER",
            Self::Fighter => "FIGHTER",
            Self::Fan => "FAN",
            Self::Superball => "SUPERBALL",
            Self::Firework => "FIREWORK",
            Self::C4 => "C4",
            Self::Virus => "VIRUS",
            Self::Nitro => "NITRO",
            Self::Ant => "ANT",
            Self::Torch => "TORCH",
            Self::Soapy => "SOAPY",
            Self::Thunder => "THUNDER",
            Self::Bomb => "BOMB",
            Self::Laser => "LASER",
            Self::Vine => "VINE",
            Self::Salt => "SALT",
            Self::SaltWater => "SALTWATER",
            Self::Glass => "GLASS",
            Self::Bird => "BIRD",
            Self::Mercury => "MERCURY",
            Self::Fuse => "FUSE",
            Self::Cloud => "CLOUD",
            Self::Pump => "PUMP",
            Self::Snow => "SNOW",
            Self::Wind => "WIND",
            Self::Air => "AIR",
            Self::Drag => "DRAG",
            Self::Wheel => "WHEEL",
            Self::Box => "BOX",
            Self::Bubble => "BUBBLE",
            Self::Ball => "BALL",
            Self::Create => "CREATE",
        }
    }
    pub fn color(self) -> u32 {
        match self {
            Self::Empty => 0x080b15,
            Self::Sand => 0xf6da66,
            Self::Water => 0x277dff,
            Self::Fire => 0xff6138,
            Self::Seed => 0x61fa67,
            Self::Wood => 0x9a643c,
            Self::Gunpowder => 0xb7aaab,
            Self::Oil => 0xb08146,
            Self::Stone => 0x858b9f,
            Self::Magma => 0xff321b,
            Self::Ice => 0x95edff,
            Self::Steam => 0x9ca7c7,
            Self::Acid => 0xc7ff27,
            Self::Metal => 0xd4dcf1,
            Self::Spark => 0xffff9c,
            Self::Gas => 0xd77dff,
            Self::Clone => 0xff5bd1,
            Self::Wall => 0x626c81,
            Self::Player => 0x71ffb7,
            Self::Fighter => 0xff6dae,
            Self::Fan => 0x46d5ff,
            Self::Superball => 0xdc58ff,
            Self::Firework => 0xffb847,
            Self::C4 => 0xd1aa66,
            Self::Virus => 0xca53ff,
            Self::Nitro => 0xffbd55,
            Self::Ant => 0xdb702e,
            Self::Torch => 0xff7540,
            Self::Soapy => 0x81ffff,
            Self::Thunder => 0xffffcc,
            Self::Bomb => 0x7f899a,
            Self::Laser => 0xff365d,
            Self::Vine => 0x32bf64,
            Self::Salt => 0xffffff,
            Self::SaltWater => 0x507bcb,
            Self::Glass => 0x72bbbe,
            Self::Bird => 0xa2f0fa,
            Self::Mercury => 0xc6d1e1,
            Self::Fuse => 0xe7943d,
            Self::Cloud => 0xb8c9e8,
            Self::Pump => 0x73acae,
            Self::Snow => 0xe5faff,
            Self::Wind => 0x59b7ff,
            Self::Air => 0x86e1e9,
            Self::Drag => 0xeaeaea,
            Self::Wheel => 0xb7a8ff,
            Self::Box => 0xd5a66e,
            Self::Bubble => 0x78e5e5,
            Self::Ball => 0xff82ae,
            Self::Create => 0xf9a5e1,
        }
    }
    pub fn is_tool(self) -> bool {
        matches!(self, Self::Wind | Self::Air | Self::Drag | Self::Create)
    }
    pub fn is_object(self) -> bool {
        matches!(self, Self::Wheel | Self::Box | Self::Bubble | Self::Ball)
    }
    pub fn is_cell(self) -> bool {
        !self.is_tool() && !self.is_object() && !matches!(self, Self::Player | Self::Fighter)
    }
    #[inline(always)]
    fn density(self) -> u8 {
        match self {
            Self::Empty => 0,
            Self::Fire | Self::Steam | Self::Gas | Self::Spark | Self::Cloud | Self::Bird => 1,
            Self::Oil | Self::Nitro => 2,
            Self::Water | Self::Acid | Self::Soapy | Self::SaltWater => 3,
            Self::Sand
            | Self::Seed
            | Self::Gunpowder
            | Self::Salt
            | Self::Snow
            | Self::Superball
            | Self::Firework
            | Self::Bomb
            | Self::Virus
            | Self::Ant => 5,
            Self::Stone | Self::Mercury => 6,
            Self::Magma => 4,
            _ => 255,
        }
    }
    fn combustible(self) -> bool {
        matches!(
            self,
            Self::Seed
                | Self::Wood
                | Self::Gunpowder
                | Self::Oil
                | Self::Gas
                | Self::Nitro
                | Self::C4
                | Self::Firework
                | Self::Bomb
                | Self::Vine
                | Self::Fuse
                | Self::Ant
                | Self::Bird
        )
    }
    #[inline(always)]
    fn movable(self) -> bool {
        self.density() < 255
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cell {
    pub material: Material,
    pub life: u8,
}
impl Cell {
    pub fn new(material: Material) -> Self {
        Self {
            material,
            life: match material {
                Material::Fire => 45,
                Material::Spark => 8,
                Material::Steam => 180,
                Material::Thunder => 14,
                Material::Laser => 63,
                Material::Firework => Material::Sand as u8,
                Material::Virus => 160,
                _ => 0,
            },
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
}

#[path = "actors.rs"]
mod actors;
pub use actors::Actor;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Motion {
    pub vx: f32,
    pub vy: f32,
}

struct MotionStep {
    vx: f32,
    vy: f32,
    steps: i32,
    fraction: f32,
}

// A chain reaction is spread across updates instead of freezing input in one giant tick.
const MAX_BLASTS_PER_STEP: usize = 64;
struct BlastPoint {
    dx: i32,
    dy: i32,
    vx: f32,
    vy: f32,
    ray: Vec<(i32, i32)>,
}
fn make_blast_stencil(radius: i32) -> Vec<BlastPoint> {
    let mut points = Vec::new();
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy > radius * radius {
                continue;
            }
            let distance = ((dx * dx + dy * dy) as f32).sqrt();
            let force = (1. - distance / (radius as f32 + 1.)) * 9.;
            let steps = dx.abs().max(dy.abs());
            points.push(BlastPoint {
                dx,
                dy,
                vx: dx as f32 / distance.max(1.) * force,
                vy: dy as f32 / distance.max(1.) * force,
                ray: (1..=steps)
                    .map(|n| (dx * n / steps, dy * n / steps))
                    .collect(),
            });
        }
    }
    points
}
fn permanent_barrier(m: Material) -> bool {
    matches!(m, Material::Wall | Material::Metal | Material::Clone)
}

pub struct World {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
    pub actors: Vec<Actor>,
    pub air: Air,
    pub bodies: Vec<Body>,
    pub brush_payload: Material,
    pub(crate) motion: Vec<Motion>,
    visited: Vec<u32>,
    tick: u32,
    blasts_this_step: usize,
    wind_quiet: bool,
    #[cfg(test)]
    reference_steps: bool,
    rng: u64,
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width >= 16 && height >= 16 && width.checked_mul(height).is_some());
        Self {
            width,
            height,
            cells: vec![Cell::default(); width * height],
            actors: Vec::new(),
            air: Air::new(width, height),
            bodies: Vec::new(),
            brush_payload: Material::Sand,
            motion: vec![Motion::default(); width * height],
            visited: vec![0; width * height],
            tick: 0,
            blasts_this_step: 0,
            wind_quiet: false,
            #[cfg(test)]
            reference_steps: false,
            rng: 0x69d2_739b_b853_a451,
        }
    }
    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
        self.motion.fill(Motion::default());
        self.actors.clear();
        self.air.clear();
        self.bodies.clear();
    }
    pub fn seed_demo(&mut self) {
        self.clear();
        let w = self.width as i32;
        let h = self.height as i32;
        self.paint_line(0, h - 8, w - 1, h - 8, 2, Material::Wall);
        self.paint_line(w / 8, h * 2 / 3, w / 3, h * 2 / 3, 2, Material::Wall);
        self.paint_line(
            w * 3 / 5,
            h * 3 / 4,
            w * 4 / 5,
            h * 3 / 4,
            2,
            Material::Wall,
        );
        self.paint_line(w / 8, h * 2 / 3, w / 8, h - 9, 2, Material::Wall);
        self.paint_line(w / 3, h * 2 / 3, w / 3, h - 9, 2, Material::Wall);
        for y in h * 4 / 5..h - 10 {
            self.paint_line(w / 8 + 4, y, w / 3 - 4, y, 0, Material::Water);
        }
        self.paint_line(w / 5, 32, w / 5 + 20, 48, 5, Material::Sand);
        self.paint_line(w * 2 / 5, h - 14, w / 2, h - 14, 4, Material::Sand);
        self.paint_line(
            w * 3 / 5,
            h * 3 / 4 - 4,
            w * 4 / 5,
            h * 3 / 4 - 4,
            1,
            Material::Seed,
        );
        self.paint_line(
            w * 3 / 5,
            h * 3 / 4 - 7,
            w * 4 / 5,
            h * 3 / 4 - 7,
            0,
            Material::Water,
        );
        self.paint_line(
            w * 9 / 10,
            h - 15,
            w * 9 / 10 + 10,
            h - 15,
            5,
            Material::Magma,
        );
        self.paint_line(w / 2, h - 18, w / 2, h - 18, 0, Material::Player);
        self.paint_line(w * 3 / 4, h - 18, w * 3 / 4, h - 18, 0, Material::Fighter);
        if w >= 64 && h >= 64 {
            let radius = (w.min(h) / 25).clamp(3, 12) as f32;
            self.bodies.push(Body::new(
                BodyKind::Wheel,
                w as f32 * 0.45,
                h as f32 * 0.50,
                radius,
            ));
            self.bodies.push(Body::new(
                BodyKind::Box,
                w as f32 * 0.65,
                h as f32 * 0.30,
                radius * 0.7,
            ));
            self.bodies.push(Body::new(
                BodyKind::Bubble,
                w as f32 * 0.75,
                h as f32 * 0.50,
                radius * 0.8,
            ));
            // A small persistent fan drives the wheel without flooding the scene.
            let fan_x = (w * 45 / 100 - radius as i32 - 10).max(2);
            self.paint_line(fan_x, h / 2 - 3, fan_x, h / 2 - 3, 1, Material::Fan);
            self.paint_line(w * 7 / 8, h / 8, w * 7 / 8 + 3, h / 8, 1, Material::Cloud);
            self.paint_line(w / 5, h * 3 / 4, w / 5, h * 3 / 4, 1, Material::Salt);
        }
    }
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        use std::io::Write;
        let path = path.as_ref();
        // Keep the last legacy scene recoverable when upgrading its on-disk format.
        if let Ok(mut old) = std::fs::File::open(path) {
            use std::io::Read;
            let mut header = [0; 9];
            if old.read_exact(&mut header).is_ok() && &header[..8] == b"SURFDUST" && header[8] < 3 {
                let mut backup = path.as_os_str().to_os_string();
                backup.push(format!(".v{}.bak", header[8]));
                if !std::path::Path::new(&backup).exists() {
                    std::fs::copy(path, backup)?;
                }
            }
        }
        let mut bytes = Vec::with_capacity(self.cells.len() * 10 + 64 + self.actors.len() * 200);
        bytes.extend_from_slice(b"SURFDUST\x03");
        bytes.extend_from_slice(&(self.width as u32).to_le_bytes());
        bytes.extend_from_slice(&(self.height as u32).to_le_bytes());
        bytes.extend_from_slice(&self.tick.to_le_bytes());
        bytes.extend_from_slice(&self.rng.to_le_bytes());
        for c in &self.cells {
            bytes.extend_from_slice(&[c.material as u8, c.life]);
        }
        bytes.push(self.actors.len() as u8);
        for a in &self.actors {
            for v in [a.x, a.y, a.vx, a.vy] {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            bytes.extend_from_slice(&[a.kind as u8, a.health]);
            bytes.extend_from_slice(&[a.dead as u8, a.stride, a.jump_wait, a.knocked]);
            bytes.extend_from_slice(&a.age.to_le_bytes());
            for joint in &a.joints {
                for v in [joint.x, joint.y, joint.old_x, joint.old_y] {
                    bytes.extend_from_slice(&v.to_le_bytes());
                }
            }
        }
        for node in &self.air.nodes {
            for v in [node.vx, node.vy, node.pressure] {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&(self.bodies.len() as u16).to_le_bytes());
        for b in &self.bodies {
            bytes.push(b.kind as u8);
            for v in [b.x, b.y, b.vx, b.vy, b.angle, b.angular_velocity, b.radius] {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            bytes.extend_from_slice(&b.life.to_le_bytes());
            bytes.push(b.payload as u8);
        }
        bytes.push(self.brush_payload as u8);
        for motion in &self.motion {
            bytes.extend_from_slice(&motion.vx.to_le_bytes());
            bytes.extend_from_slice(&motion.vy.to_le_bytes());
        }
        let mut tmp = path.as_os_str().to_os_string();
        tmp.push(".tmp");
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        std::fs::rename(&tmp, path)
    }
    pub fn load(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        use std::io::{Error, ErrorKind, Read};
        fn invalid() -> Error {
            Error::new(
                ErrorKind::InvalidData,
                "Invalid or unsupported sandbox save",
            )
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(24_000_001)
            .read_to_end(&mut bytes)?;
        let mut cursor = 0;
        fn take<'a>(bytes: &'a [u8], cursor: &mut usize, n: usize) -> std::io::Result<&'a [u8]> {
            let part = bytes
                .get(*cursor..*cursor + n)
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Truncated sandbox save"))?;
            *cursor += n;
            Ok(part)
        }
        let header = take(&bytes, &mut cursor, 9)?;
        if &header[..8] != b"SURFDUST" || ![1, 2, 3].contains(&header[8]) {
            return Err(invalid());
        }
        let version = header[8];
        let width = u32::from_le_bytes(take(&bytes, &mut cursor, 4)?.try_into().unwrap()) as usize;
        let height = u32::from_le_bytes(take(&bytes, &mut cursor, 4)?.try_into().unwrap()) as usize;
        if !(16..=2048).contains(&width)
            || !(16..=2048).contains(&height)
            || width * height > 2_000_000
        {
            return Err(invalid());
        }
        let tick = u32::from_le_bytes(take(&bytes, &mut cursor, 4)?.try_into().unwrap());
        let rng = u64::from_le_bytes(take(&bytes, &mut cursor, 8)?.try_into().unwrap());
        let mut world = Self::new(width, height);
        world.tick = tick;
        world.rng = rng.max(1);
        for c in &mut world.cells {
            let b = take(&bytes, &mut cursor, 2)?;
            let material = Material::from_u8(b[0]).ok_or_else(invalid)?;
            if !material.is_cell()
                || (material == Material::Clone
                    && !Material::from_u8(b[1]).is_some_and(Material::is_cell))
                || (material == Material::Firework
                    && !Material::from_u8(b[1] & 63).is_some_and(Material::is_cell))
            {
                return Err(invalid());
            }
            *c = Cell {
                material,
                life: b[1],
            };
        }
        let count = take(&bytes, &mut cursor, 1)?[0];
        if count > 64 {
            return Err(invalid());
        }
        for _ in 0..count {
            let mut values = [0.; 4];
            for v in &mut values {
                *v = f32::from_le_bytes(take(&bytes, &mut cursor, 4)?.try_into().unwrap());
                if !v.is_finite() {
                    return Err(invalid());
                }
            }
            let b = take(&bytes, &mut cursor, 2)?;
            let kind = Material::from_u8(b[0]).ok_or_else(invalid)?;
            if !matches!(kind, Material::Player | Material::Fighter)
                || values[0] < 0.
                || values[0] >= width as f32
                || values[1] < 0.
                || values[1] >= height as f32
                || values[2].abs() > 100.
                || values[3].abs() > 100.
            {
                return Err(invalid());
            }
            let mut actor = Actor::new(values[0], values[1], kind);
            actor.impulse(values[2], values[3]);
            actor.health = b[1];
            if version >= 3 {
                let state = take(&bytes, &mut cursor, 4)?;
                if state[0] > 1 {
                    return Err(invalid());
                }
                actor.dead = state[0] != 0;
                actor.stride = state[1];
                actor.jump_wait = state[2];
                actor.knocked = state[3];
                actor.age = u16::from_le_bytes(take(&bytes, &mut cursor, 2)?.try_into().unwrap());
                for joint in &mut actor.joints {
                    let mut v = [0.; 4];
                    for value in &mut v {
                        *value =
                            f32::from_le_bytes(take(&bytes, &mut cursor, 4)?.try_into().unwrap());
                        if !value.is_finite() || value.abs() > 4096. {
                            return Err(invalid());
                        }
                    }
                    *joint = actors::Joint {
                        x: v[0],
                        y: v[1],
                        old_x: v[2],
                        old_y: v[3],
                    };
                }
            }
            world.actors.push(actor);
        }
        if version >= 2 {
            for node in &mut world.air.nodes {
                let mut values = [0.; 3];
                for v in &mut values {
                    *v = f32::from_le_bytes(take(&bytes, &mut cursor, 4)?.try_into().unwrap());
                    if !v.is_finite() || v.abs() > 1000. {
                        return Err(invalid());
                    }
                }
                node.vx = values[0];
                node.vy = values[1];
                node.pressure = values[2];
            }
            let count =
                u16::from_le_bytes(take(&bytes, &mut cursor, 2)?.try_into().unwrap()) as usize;
            if count > MAX_BODIES {
                return Err(invalid());
            }
            for _ in 0..count {
                let kind =
                    BodyKind::from_u8(take(&bytes, &mut cursor, 1)?[0]).ok_or_else(invalid)?;
                let mut v = [0.; 7];
                for value in &mut v {
                    *value = f32::from_le_bytes(take(&bytes, &mut cursor, 4)?.try_into().unwrap());
                }
                let life = u16::from_le_bytes(take(&bytes, &mut cursor, 2)?.try_into().unwrap());
                let payload = Material::from_u8(take(&bytes, &mut cursor, 1)?[0])
                    .filter(|m| m.is_cell())
                    .ok_or_else(invalid)?;
                let body = Body {
                    kind,
                    x: v[0],
                    y: v[1],
                    vx: v[2],
                    vy: v[3],
                    angle: v[4],
                    angular_velocity: v[5],
                    radius: v[6],
                    life,
                    payload,
                };
                if !body.valid(width, height) {
                    return Err(invalid());
                }
                world.bodies.push(body);
            }
            world.brush_payload = Material::from_u8(take(&bytes, &mut cursor, 1)?[0])
                .filter(|m| m.is_cell())
                .ok_or_else(invalid)?;
        }
        if version >= 3 {
            for motion in &mut world.motion {
                motion.vx = f32::from_le_bytes(take(&bytes, &mut cursor, 4)?.try_into().unwrap());
                motion.vy = f32::from_le_bytes(take(&bytes, &mut cursor, 4)?.try_into().unwrap());
                if !motion.vx.is_finite()
                    || !motion.vy.is_finite()
                    || motion.vx.abs() > 32.
                    || motion.vy.abs() > 32.
                {
                    return Err(invalid());
                }
            }
        }
        if cursor != bytes.len() {
            return Err(invalid());
        }
        for node in &mut world.air.nodes {
            node.vx = clean_residue(node.vx);
            node.vy = clean_residue(node.vy);
            node.pressure = clean_residue(node.pressure);
        }
        for motion in &mut world.motion {
            motion.vx = clean_residue(motion.vx);
            motion.vy = clean_residue(motion.vy);
        }
        Ok(world)
    }
    #[inline(always)]
    fn random(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 16) as u32
    }
    fn fast_paths(&self) -> bool {
        #[cfg(test)]
        if self.reference_steps {
            return false;
        }
        true
    }
    #[inline(always)]
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if (x as usize) < self.width && (y as usize) < self.height {
            Some(y as usize * self.width + x as usize)
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn at(&self, x: i32, y: i32) -> Material {
        self.index(x, y)
            .map(|i| self.cells[i].material)
            .unwrap_or(Material::Wall)
    }
    fn set(&mut self, x: i32, y: i32, material: Material) {
        if let Some(i) = self.index(x, y) {
            self.cells[i] = Cell::new(material);
            self.motion[i] = Motion::default();
            self.visited[i] = self.tick;
        }
    }
    pub fn paint_line(
        &mut self,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        radius: i32,
        material: Material,
    ) {
        let radius = radius.clamp(0, 48);
        let (x1, y1) = if material == Material::Wind {
            (
                x1.clamp(0, self.width as i32 - 1),
                y1.clamp(0, self.height as i32 - 1),
            )
        } else {
            (x1, y1)
        };
        if self.index(x1, y1).is_none() {
            return;
        }
        if material == Material::Drag {
            self.drag_at(x1, y1, x1 - x0, y1 - y0, radius);
            return;
        }
        if material == Material::Wind {
            let dx = (x1 - x0) as f32;
            let dy = (y1 - y0) as f32;
            let len = dx.hypot(dy).max(1.);
            self.air.add_wind(
                x1 as f32,
                y1 as f32,
                if dx == 0. && dy == 0. {
                    3.
                } else {
                    dx / len * 3.
                },
                dy / len * 3.,
                radius.max(5) as f32,
            );
            return;
        }
        if material == Material::Air {
            self.air
                .add_pressure(x1 as f32, y1 as f32, 4., radius.max(5) as f32);
            return;
        }
        if material.is_object() {
            let kind = match material {
                Material::Wheel => BodyKind::Wheel,
                Material::Box => BodyKind::Box,
                Material::Bubble => BodyKind::Bubble,
                _ => BodyKind::Ball,
            };
            if self.bodies.len() < MAX_BODIES
                && !self.bodies.iter().any(|b| {
                    (b.x - x1 as f32).hypot(b.y - y1 as f32) < b.radius + radius.max(3) as f32
                })
            {
                let mut body = Body::new(kind, x1 as f32, y1 as f32, radius.clamp(3, 48) as f32);
                if kind == BodyKind::Ball {
                    body.payload = self.brush_payload;
                }
                self.bodies.push(body);
            }
            return;
        }
        let material = if material == Material::Create {
            Material::Clone
        } else {
            material
        };
        if matches!(material, Material::Player | Material::Fighter) {
            if self.index(x1, y1).is_some()
                && self.actors.len() < 64
                && !self
                    .actors
                    .iter()
                    .any(|a| (a.x - x1 as f32).abs() < 6. && (a.y - y1 as f32).abs() < 12.)
            {
                self.actors.push(Actor::new(x1 as f32, y1 as f32, material));
            }
            return;
        }
        let steps = (x1 - x0).abs().max((y1 - y0).abs()).clamp(1, 4096);
        for s in 0..=steps {
            let x = x0 + (x1 - x0) * s / steps;
            let y = y0 + (y1 - y0) * s / steps;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if dx * dx + dy * dy <= radius * radius {
                        if let Some(i) = self.index(x + dx, y + dy) {
                            if material == Material::Empty
                                || self.cells[i].material == Material::Empty
                            {
                                self.cells[i] = Cell::new(material);
                                self.motion[i] = Motion::default();
                                if material == Material::Firework {
                                    self.cells[i].life = self.brush_payload as u8 & 63;
                                } else if matches!(material, Material::Fan | Material::Pump) {
                                    let dx = x1 - x0;
                                    let dy = y1 - y0;
                                    self.cells[i].life = if dx.abs() >= dy.abs() {
                                        if dx < 0 {
                                            2
                                        } else {
                                            0
                                        }
                                    } else if dy > 0 {
                                        1
                                    } else {
                                        3
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }
        if material == Material::Empty {
            self.bodies
                .retain(|b| (b.x - x1 as f32).hypot(b.y - y1 as f32) > b.radius + radius as f32);
            self.actors.retain(|a| {
                (a.x - x1 as f32).abs() > radius as f32 + 3.
                    || (a.y - y1 as f32).abs() > radius as f32 + 8.
            });
        }
    }
    pub fn drag_at(&mut self, x: i32, y: i32, dx: i32, dy: i32, radius: i32) {
        let dx = dx.clamp(-16, 16);
        let dy = dy.clamp(-16, 16);
        let r = radius.clamp(2, 40);
        for a in &mut self.actors {
            if (a.x - x as f32).hypot(a.y - y as f32) < r as f32 + 8. {
                a.x = (a.x + dx as f32).clamp(3., self.width as f32 - 4.);
                a.y = (a.y + dy as f32).clamp(5., self.height as f32 - 9.);
                a.vx = dx as f32 * 0.2;
                a.vy = dy as f32 * 0.2;
            }
        }
        for b in &mut self.bodies {
            if (b.x - x as f32).hypot(b.y - y as f32) < r as f32 + b.radius {
                if b.kind == BodyKind::Wheel {
                    let cross = (x as f32 - b.x) * dy as f32 - (y as f32 - b.y) * dx as f32;
                    b.angular_velocity = (b.angular_velocity + cross / (b.radius * b.radius) * 0.4)
                        .clamp(-0.25, 0.25);
                } else {
                    b.vx = dx as f32 * 0.4;
                    b.vy = dy as f32 * 0.4;
                }
            }
        }
        // Traverse against movement so particles cannot be dragged repeatedly in one stroke.
        for ny in 0..r * 2 + 1 {
            for nx in 0..r * 2 + 1 {
                let sx = x + if dx > 0 { r - nx } else { nx - r };
                let sy = y + if dy > 0 { r - ny } else { ny - r };
                if (sx - x).pow(2) + (sy - y).pow(2) > r * r {
                    continue;
                }
                if let (Some(i), Some(j)) = (self.index(sx, sy), self.index(sx + dx, sy + dy)) {
                    if self.cells[i].material.movable() && self.cells[j].material == Material::Empty
                    {
                        self.cells.swap(i, j);
                        self.motion.swap(i, j);
                    }
                }
            }
        }
        self.air.add_wind(
            x as f32,
            y as f32,
            dx as f32 * 0.2,
            dy as f32 * 0.2,
            r as f32,
        );
    }
    #[inline(always)]
    fn exchange(&mut self, i: usize, x: i32, y: i32, rising: bool) -> bool {
        let Some(j) = self.index(x, y) else {
            return false;
        };
        self.exchange_at(i, j, rising)
    }
    #[inline(always)]
    fn exchange_at(&mut self, i: usize, j: usize, rising: bool) -> bool {
        let a = self.cells[i].material;
        let b = self.cells[j].material;
        let allowed = if rising {
            b == Material::Empty
        } else {
            a.density() > b.density()
        };
        if allowed {
            self.cells.swap(i, j);
            self.motion.swap(i, j);
            self.visited[i] = self.tick;
            self.visited[j] = self.tick;
            true
        } else {
            false
        }
    }
    fn ignite(&mut self, x: i32, y: i32) {
        let m = self.at(x, y);
        if m == Material::Firework {
            if let Some(i) = self.index(x, y) {
                if self.cells[i].life >> 6 == 0 {
                    self.cells[i].life |= 64;
                }
            }
        } else if m == Material::Fuse {
            if let Some(i) = self.index(x, y) {
                if self.cells[i].life == 0 {
                    self.cells[i].life = 18;
                }
            }
        } else if m.combustible() {
            if matches!(
                m,
                Material::Gunpowder
                    | Material::Gas
                    | Material::Nitro
                    | Material::C4
                    | Material::Bomb
            ) {
                self.explode(x, y, 10);
            } else {
                self.set(x, y, Material::Fire);
            }
        } else if m == Material::Ice || m == Material::Snow {
            self.set(x, y, Material::Water);
        } else if m == Material::Water {
            self.set(x, y, Material::Steam);
        }
    }
    fn explode(&mut self, x: i32, y: i32, r: i32) {
        let radius = r as f32;
        let source = self.at(x, y);
        if source == Material::Bomb {
            self.bomb_collapse(x, y);
            return;
        }
        if self.blasts_this_step >= MAX_BLASTS_PER_STEP {
            // These four materials otherwise have no use for life=1. It survives saves.
            if let Some(i) = self.index(x, y) {
                self.cells[i].life = 1;
                self.visited[i] = self.tick;
            }
            return;
        }
        self.blasts_this_step += 1;
        self.air.add_pressure(
            x as f32,
            y as f32,
            if source == Material::Gas {
                2.
            } else if source == Material::C4 {
                1.
            } else {
                10.
            },
            radius,
        );
        static STENCIL: std::sync::OnceLock<Vec<BlastPoint>> = std::sync::OnceLock::new();
        let fallback;
        let points = if r == 10 {
            STENCIL.get_or_init(|| make_blast_stencil(10)).as_slice()
        } else {
            fallback = make_blast_stencil(r);
            &fallback
        };
        // Most blasts are in open air. Only trace rays if this region contains a shield.
        let x0 = (x - r).max(0) as usize;
        let x1 = (x + r + 1).min(self.width as i32).max(0) as usize;
        let y0 = (y - r).max(0) as usize;
        let y1 = (y + r + 1).min(self.height as i32).max(0) as usize;
        let shielded = x < r
            || y < r
            || x + r >= self.width as i32
            || y + r >= self.height as i32
            || (y0..y1).any(|row| {
                self.cells[row * self.width + x0..row * self.width + x1]
                    .iter()
                    .any(|c| permanent_barrier(c.material))
            });
        for point in points {
            let Some(i) = self.index(x + point.dx, y + point.dy) else {
                continue;
            };
            let m = self.cells[i].material;
            if m == Material::Empty {
                continue;
            }
            if shielded
                && point
                    .ray
                    .iter()
                    .any(|&(dx, dy)| permanent_barrier(self.at(x + dx, y + dy)))
            {
                continue;
            }
            if m == Material::Glass {
                self.set(x + point.dx, y + point.dy, Material::Sand);
            } else if m == Material::Gas && source == Material::Gas {
                if self.random() % 100 == 0 {
                    self.set(x + point.dx, y + point.dy, Material::Fire);
                }
            } else if m == Material::C4 && point.dx.abs() <= 1 && point.dy.abs() <= 1 {
                self.cells[i].life = 1;
            }
            if source != Material::Gas && self.cells[i].material.movable() {
                self.motion[i].vx = (self.motion[i].vx + point.vx).clamp(-24., 24.);
                self.motion[i].vy = (self.motion[i].vy + point.vy).clamp(-24., 24.);
            }
        }
        self.set(
            x,
            y,
            if source == Material::C4 {
                Material::Empty
            } else {
                Material::Fire
            },
        );
        for body in &mut self.bodies {
            let dx = body.x - x as f32;
            let dy = body.y - y as f32;
            let d = dx.hypot(dy).max(1.);
            if d < radius * 2. && body.kind != BodyKind::Wheel {
                let force = (1. - d / (radius * 2.)) * 7.;
                body.vx += dx / d * force;
                body.vy += dy / d * force;
                body.angular_velocity += dx.signum() * force * 0.02;
            }
        }
        for actor in &mut self.actors {
            let dx = actor.x - x as f32;
            let dy = actor.y - y as f32;
            let d = dx.hypot(dy).max(1.);
            if d < radius * 2. {
                let force = (1. - d / (radius * 2.)) * 8.;
                actor.impulse(dx / d * force, dy / d * force);
                actor.health = actor.health.saturating_sub((force * 24.) as u8);
            }
        }
    }
    fn bomb_collapse(&mut self, x: i32, y: i32) {
        // Original BOMB grabs a small cloud inward before its tagged grains decay.
        for dy in -4_i32..=4 {
            for dx in -4_i32..=4 {
                if dx * dx + dy * dy > 16 || !self.blast_visible(x, y, x + dx, y + dy) {
                    continue;
                }
                let Some(i) = self.index(x + dx, y + dy) else {
                    continue;
                };
                let m = self.cells[i].material;
                if !m.movable() || matches!(m, Material::Empty | Material::Bomb) {
                    continue;
                }
                self.set(x + dx, y + dy, Material::Bomb);
                self.cells[i].life = 128 | m as u8;
                self.motion[i] = Motion {
                    vx: -dx as f32,
                    vy: -dy as f32,
                };
            }
        }
        self.set(x, y, Material::Empty);
    }
    fn blast_visible(&self, x: i32, y: i32, tx: i32, ty: i32) -> bool {
        let steps = (tx - x).abs().max((ty - y).abs());
        (1..=steps).all(|n| {
            !matches!(
                self.at(x + (tx - x) * n / steps, y + (ty - y) * n / steps),
                Material::Wall | Material::Metal | Material::Clone
            )
        })
    }
    #[inline(always)]
    fn integrate_motion(&mut self, i: usize, x: i32, y: i32, coupling: f32) -> Option<MotionStep> {
        if self.wind_quiet && self.motion[i].vx == 0. && self.motion[i].vy == 0. {
            self.motion[i] = Motion::default();
            return None;
        }
        let (ax, ay) = if self.wind_quiet {
            (0., 0.)
        } else {
            self.air.sample_pixel(x as usize, y as usize)
        };
        let v = &mut self.motion[i];
        v.vx = (v.vx * 0.96 + (ax - v.vx) * coupling).clamp(-24., 24.);
        v.vy = (v.vy * 0.96 + (ay - v.vy) * coupling).clamp(-24., 24.);
        v.vx = clean_residue(v.vx);
        v.vy = clean_residue(v.vy);
        let (raw_x, raw_y) = (v.vx, v.vy);
        let fast = self.fast_paths();
        let (vx, vy) = if fast && raw_x.abs() + raw_y.abs() <= 3.5 {
            // This conservative L1 bound is below the 3.8-pixel length cap.
            // Avoid squaring tiny currents when the norm cannot change the result.
            (raw_x, raw_y)
        } else {
            // One component is at least 1.75 here. A component below 1e-18
            // cannot affect its squared sum at f32 precision; omitting that
            // product also avoids generating subnormal intermediate results.
            let xx = if fast && raw_x.abs() < 1e-18 {
                0.
            } else {
                raw_x * raw_x
            };
            let yy = if fast && raw_y.abs() < 1e-18 {
                0.
            } else {
                raw_y * raw_y
            };
            let speed_squared = xx + yy;
            if speed_squared > 3.8 * 3.8 {
                let scale = 3.8 / speed_squared.sqrt();
                (raw_x * scale, raw_y * scale)
            } else {
                (raw_x, raw_y)
            }
        };
        let distance = vx.abs().max(vy.abs());
        let steps = distance.ceil() as i32;
        if steps == 0 {
            return None;
        }
        let fraction = (self.random() & 65535) as f32 / 65536.;
        Some(MotionStep {
            vx,
            vy,
            steps,
            fraction,
        })
    }
    #[inline(always)]
    fn enclosed_motion(&mut self, i: usize, step: MotionStep) {
        let (dx, dy) = if step.steps == 1 {
            (step.vx + step.fraction, step.vy + step.fraction)
        } else {
            (
                step.vx * step.steps as f32 / step.steps as f32 + step.fraction,
                step.vy * step.steps as f32 / step.steps as f32 + step.fraction,
            )
        };
        if !(0. ..1.).contains(&dx) || !(0. ..1.).contains(&dy) {
            self.motion[i].vx *= 0.15;
            self.motion[i].vy *= 0.15;
        }
    }
    #[inline(always)]
    fn uniformly_enclosed(&self, i: usize, x: i32, y: i32, material: Material) -> bool {
        x > 0
            && y > 0
            && x + 1 < self.width as i32
            && y + 1 < self.height as i32
            && [
                i - 1,
                i + 1,
                i - self.width,
                i + self.width,
                i + self.width - 1,
                i + self.width + 1,
            ]
            .iter()
            .all(|&j| self.cells[j].material == material)
    }
    fn transport_particle(
        &mut self,
        mut i: usize,
        mut x: i32,
        mut y: i32,
        m: Material,
    ) -> (usize, i32, i32) {
        if !m.movable() || (m == Material::Spark && self.cells[i].life & 128 != 0) {
            self.motion[i] = Motion::default();
            return (i, x, y);
        }
        let coupling = if m.density() <= 1 {
            0.20
        } else if m == Material::Mercury || m == Material::Stone {
            0.015
        } else {
            0.075
        };
        let Some(step) = self.integrate_motion(i, x, y, coupling) else {
            return (i, x, y);
        };
        if self.fast_paths()
            && x > 0
            && y > 0
            && x + 1 < self.width as i32
            && y + 1 < self.height as i32
            && [i - 1, i + 1, i - self.width, i + self.width]
                .iter()
                .all(|&j| self.cells[j].material != Material::Empty)
        {
            self.enclosed_motion(i, step);
            return (i, x, y);
        }
        let MotionStep {
            vx,
            vy,
            steps,
            fraction,
        } = step;
        let start_x = x;
        let start_y = y;
        for n in 1..=steps {
            let tx = start_x + (vx * n as f32 / steps as f32 + fraction).floor() as i32;
            let ty = start_y + (vy * n as f32 / steps as f32 + fraction).floor() as i32;
            if tx == x && ty == y {
                continue;
            }
            let Some(j) = self.index(tx, ty) else {
                self.motion[i] = Motion::default();
                break;
            };
            if self.cells[j].material != Material::Empty
                || (tx != x
                    && ty != y
                    && (self.at(tx, y) != Material::Empty || self.at(x, ty) != Material::Empty))
            {
                self.motion[i].vx *= 0.15;
                self.motion[i].vy *= 0.15;
                break;
            }
            self.cells.swap(i, j);
            self.motion.swap(i, j);
            self.visited[i] = self.tick;
            self.visited[j] = self.tick;
            i = j;
            x = tx;
            y = ty;
        }
        (i, x, y)
    }
    pub fn step(&mut self, input: Input) {
        self.advance::<false>(input);
    }
    pub fn step_profiled(&mut self, input: Input) -> [std::time::Duration; 4] {
        self.advance::<true>(input)
    }
    fn advance<const PROFILE: bool>(&mut self, input: Input) -> [std::time::Duration; 4] {
        self.blasts_this_step = 0;
        self.tick = self.tick.wrapping_add(1);
        if self.tick == 0 {
            self.visited.fill(0);
            self.tick = 1;
        }
        let start = PROFILE.then(std::time::Instant::now);
        for motion in &mut self.motion {
            motion.vx = clean_residue(motion.vx);
            motion.vy = clean_residue(motion.vy);
        }
        self.air.step(self.width, self.height, &self.cells);
        self.wind_quiet =
            self.fast_paths() && self.air.nodes.iter().all(|n| n.vx == 0. && n.vy == 0.);
        let air_done = PROFILE.then(std::time::Instant::now);
        for y in (0..self.height as i32).rev() {
            let backwards = self.random() & 1 == 0;
            for n in 0..self.width {
                let x = if backwards { self.width - 1 - n } else { n } as i32;
                let i = y as usize * self.width + x as usize;
                if self.visited[i] == self.tick {
                    continue;
                }
                let m = self.cells[i].material;
                if m == Material::Empty {
                    continue;
                }
                if self.cells[i].life == 1
                    && matches!(
                        m,
                        Material::Gunpowder | Material::Gas | Material::C4 | Material::Nitro
                    )
                {
                    self.explode(x, y, 10);
                    continue;
                }
                let dir = if self.random() & 1 == 0 { -1 } else { 1 };
                if self.fast_paths()
                    && matches!(
                        m,
                        Material::Water
                            | Material::Oil
                            | Material::Sand
                            | Material::Gunpowder
                            | Material::Stone
                            | Material::Seed
                            | Material::Salt
                            | Material::SaltWater
                            | Material::Mercury
                    )
                    && (!matches!(m, Material::SaltWater | Material::Mercury)
                        || self.cells[i].life == 0)
                    && self.uniformly_enclosed(i, x, y, m)
                {
                    let coupling = if matches!(m, Material::Stone | Material::Mercury) {
                        0.015
                    } else {
                        0.075
                    };
                    if let Some(step) = self.integrate_motion(i, x, y, coupling) {
                        self.enclosed_motion(i, step);
                    }
                    continue;
                }
                let (i, x, y) = self.transport_particle(i, x, y, m);
                let neighbors = [(x, y - 1), (x + dir, y), (x - dir, y), (x, y + 1)];
                if self.step_new_material(i, x, y, m, dir, &neighbors) {
                    continue;
                }
                match m {
                    Material::Fire | Material::Spark => {
                        // High bit marks a spark travelling through a metal cell.
                        // Keep the conductor in place and restore it after the pulse.
                        let conductor = m == Material::Spark && self.cells[i].life & 128 != 0;
                        if self.cells[i].life & 127 <= 1 {
                            self.set(
                                x,
                                y,
                                if conductor {
                                    Material::Metal
                                } else {
                                    Material::Empty
                                },
                            );
                            continue;
                        }
                        self.cells[i].life -= 1;
                        for (nx, ny) in neighbors {
                            if m == Material::Fire || self.at(nx, ny).combustible() {
                                self.ignite(nx, ny);
                            }
                            if m == Material::Spark
                                && matches!(
                                    self.at(nx, ny),
                                    Material::Metal | Material::Mercury | Material::SaltWater
                                )
                                && self.random() % 5 == 0
                            {
                                if self.at(nx, ny) == Material::Metal {
                                    self.set(nx, ny, Material::Spark);
                                }
                                if let Some(j) = self.index(nx, ny) {
                                    self.cells[j].life = 128 | 8;
                                }
                            }
                        }
                        if self.cells[i].material != m {
                            continue;
                        }
                        if conductor {
                            continue;
                        }
                        if self.exchange(i, x + dir, y - 1, true) {
                            continue;
                        }
                        self.exchange(i, x, y - 1, true);
                    }
                    Material::Steam | Material::Gas => {
                        if m == Material::Steam {
                            if self.cells[i].life == 0 {
                                self.set(x, y, Material::Water);
                                continue;
                            }
                            self.cells[i].life -= 1;
                        }
                        if self.exchange(i, x, y - 1, true)
                            || self.exchange(i, x + dir, y - 1, true)
                        {
                            continue;
                        }
                        if self.random() & 1 == 0 {
                            self.exchange(i, x + dir, y, true);
                        }
                    }
                    Material::Magma => {
                        for (nx, ny) in neighbors {
                            if self.at(nx, ny) == Material::Water {
                                self.set(nx, ny, Material::Steam);
                                self.set(x, y, Material::Stone);
                                break;
                            }
                            if self.random() % 3 == 0 {
                                self.ignite(nx, ny);
                            }
                        }
                        if self.cells[i].material == m && self.random() & 1 == 0 {
                            self.fall_liquid(i, x, y, dir);
                        }
                    }
                    Material::Acid => {
                        for (nx, ny) in neighbors {
                            let other = self.at(nx, ny);
                            if !matches!(
                                other,
                                Material::Empty
                                    | Material::Acid
                                    | Material::Wall
                                    | Material::Clone
                                    | Material::Gas
                                    | Material::Steam
                            ) && self.random() % 5 == 0
                            {
                                self.set(nx, ny, Material::Empty);
                                self.cells[i].life = self.cells[i].life.saturating_add(1);
                                if self.cells[i].life >= 8 {
                                    self.set(x, y, Material::Water);
                                }
                                break;
                            }
                        }
                        self.fall_liquid(i, x, y, dir);
                    }
                    Material::Water | Material::Oil => {
                        self.fall_liquid(i, x, y, dir);
                    }
                    Material::Sand | Material::Gunpowder | Material::Stone | Material::Seed => {
                        if m == Material::Seed
                            && neighbors
                                .iter()
                                .any(|&(nx, ny)| self.at(nx, ny) == Material::Water)
                            && self.random() % 12 == 0
                        {
                            self.set(x, y, Material::Wood);
                            for n in 1..=5 {
                                if self.at(x, y - n) == Material::Empty {
                                    self.set(x, y - n, Material::Seed);
                                } else {
                                    break;
                                }
                            }
                            if self.at(x - 1, y - 3) == Material::Empty {
                                self.set(x - 1, y - 3, Material::Seed);
                            }
                            if self.at(x + 1, y - 3) == Material::Empty {
                                self.set(x + 1, y - 3, Material::Seed);
                            }
                            continue;
                        }
                        if self.exchange(i, x, y + 1, false) {
                            continue;
                        }
                        if m != Material::Stone
                            && (self.exchange(i, x + dir, y + 1, false)
                                || self.exchange(i, x - dir, y + 1, false))
                        {
                            continue;
                        }
                    }
                    Material::Ice => {
                        if neighbors.iter().any(|&(nx, ny)| {
                            matches!(self.at(nx, ny), Material::Fire | Material::Magma)
                        }) {
                            self.set(x, y, Material::Water);
                        }
                    }
                    Material::Clone => {
                        if self.cells[i].life == 0 {
                            for (nx, ny) in neighbors {
                                let n = self.at(nx, ny);
                                if !matches!(
                                    n,
                                    Material::Empty
                                        | Material::Clone
                                        | Material::Wall
                                        | Material::Player
                                        | Material::Fighter
                                ) {
                                    self.cells[i].life = n as u8;
                                    break;
                                }
                            }
                        } else if let Some(copy) = Material::from_u8(self.cells[i].life) {
                            for (nx, ny) in neighbors {
                                if self.at(nx, ny) == Material::Empty && self.random() % 3 == 0 {
                                    self.set(nx, ny, copy);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        let particles_done = PROFILE.then(std::time::Instant::now);
        step_bodies(
            &mut self.bodies,
            &mut self.cells,
            self.width,
            self.height,
            &mut self.air,
        );
        let bodies_done = PROFILE.then(std::time::Instant::now);
        self.step_actors(input);
        if PROFILE {
            let end = std::time::Instant::now();
            [
                air_done.unwrap() - start.unwrap(),
                particles_done.unwrap() - air_done.unwrap(),
                bodies_done.unwrap() - particles_done.unwrap(),
                end - bodies_done.unwrap(),
            ]
        } else {
            [std::time::Duration::ZERO; 4]
        }
    }
    /// Pressure painting is separate from material painting for negative-pressure AIR strokes.
    pub fn apply_pressure(&mut self, x: i32, y: i32, radius: i32, amount: f32) {
        if amount.is_finite() {
            self.air.add_pressure(
                x as f32,
                y as f32,
                amount.clamp(-20., 20.),
                radius.clamp(3, 40) as f32,
            );
        }
    }
    fn step_new_material(
        &mut self,
        i: usize,
        x: i32,
        y: i32,
        m: Material,
        dir: i32,
        ns: &[(i32, i32); 4],
    ) -> bool {
        use Material::*;
        // The original material group has its own dispatch below; avoid four hot-neighbor
        // reads and the extended chemistry dispatch for every ordinary sand/water grain.
        if (m as u8) < Fan as u8 {
            return false;
        }
        if matches!(m, Mercury | SaltWater) && self.cells[i].life > 0 {
            self.cells[i].life = self.cells[i].life.saturating_sub(1);
            if self.cells[i].life > 128 && self.random() % 3 == 0 {
                for &(a, b) in ns {
                    match self.at(a, b) {
                        Metal => {
                            self.set(a, b, Spark);
                            let j = self.index(a, b).unwrap();
                            self.cells[j].life = 128 | 8;
                        }
                        Mercury | SaltWater => {
                            let j = self.index(a, b).unwrap();
                            if self.cells[j].life == 0 {
                                self.cells[j].life = 128 | 8;
                            }
                        }
                        _ => self.ignite(a, b),
                    }
                }
            }
            if self.cells[i].life == 128 {
                self.cells[i].life = 24;
            }
        }
        let hot = ns.iter().any(|&(nx, ny)| {
            matches!(
                self.at(nx, ny),
                Fire | Spark | Magma | Thunder | Laser | Torch
            )
        });
        match m {
            Fan => {
                self.wind_quiet = false;
                let (dx, dy) =
                    [(1., 0.), (0., 1.), (-1., 0.), (0., -1.)][(self.cells[i].life % 4) as usize];
                self.air
                    .add_wind(x as f32, y as f32, dx * 0.18, dy * 0.18, 8.);
            }
            Torch => {
                if ns
                    .iter()
                    .any(|&(a, b)| matches!(self.at(a, b), Water | SaltWater | Soapy))
                {
                    self.set(x, y, Wood);
                } else {
                    for &(a, b) in ns {
                        self.ignite(a, b);
                    }
                    if self.at(x, y - 1) == Empty && self.random() % 3 == 0 {
                        self.set(x, y - 1, Fire);
                    }
                }
            }
            C4 => {
                if hot || self.cells[i].life != 0 {
                    self.explode(x, y, 10);
                }
            }
            Fuse => {
                if self.cells[i].life == 0 && hot {
                    self.cells[i].life = 18;
                } else if self.cells[i].life > 0 {
                    self.cells[i].life -= 1;
                    if self.cells[i].life == 0 {
                        self.set(x, y, Fire);
                        for &(a, b) in ns {
                            if self.at(a, b) == Fuse {
                                let j = self.index(a, b).unwrap();
                                self.cells[j].life = 18;
                            }
                        }
                    }
                }
            }
            Thunder => {
                if self.cells[i].life == 0 {
                    self.set(x, y, Empty);
                } else {
                    self.cells[i].life -= 1;
                    for &(a, b) in ns {
                        self.ignite(a, b);
                        if matches!(self.at(a, b), Metal | Mercury | SaltWater) {
                            let j = self.index(a, b).unwrap();
                            self.cells[j].life = 128 | 8;
                            if self.at(a, b) == Metal {
                                self.cells[j].material = Spark;
                            }
                        }
                    }
                    if self.at(x + dir, y + 1) == Empty {
                        self.set(x + dir, y + 1, Thunder);
                        let j = self.index(x + dir, y + 1).unwrap();
                        self.cells[j].life = self.cells[i].life;
                    }
                }
            }
            Laser => {
                // Two heading bits encode east/south/west/north; the low bits bound ray lifetime.
                let life = self.cells[i].life;
                let heading = life >> 6;
                let ttl = life & 63;
                self.set(x, y, Empty);
                if ttl > 0 {
                    let (dx, dy) = [(1, 0), (0, 1), (-1, 0), (0, -1)][heading as usize];
                    let target = self.at(x + dx, y + dy);
                    if target == Metal {
                        if self.at(x - dx, y - dy) == Empty {
                            let reverse = (heading + 2) % 4;
                            self.set(x - dx, y - dy, Laser);
                            if let Some(j) = self.index(x - dx, y - dy) {
                                self.cells[j].life = reverse * 64 + ttl - 1;
                            }
                        }
                    } else if target == Empty || target == Glass {
                        let distance = if target == Glass { 2 } else { 1 };
                        if self.at(x + dx * distance, y + dy * distance) == Empty {
                            self.set(x + dx * distance, y + dy * distance, Laser);
                            if let Some(j) = self.index(x + dx * distance, y + dy * distance) {
                                self.cells[j].life = heading * 64 + ttl - 1;
                            }
                        }
                    } else {
                        self.ignite(x + dx, y + dy);
                        if !matches!(target, Wall | Clone | Fan | Pump) {
                            self.set(x + dx, y + dy, Fire);
                        }
                    }
                }
            }
            Pump => {
                // Direction is selected by brush stroke; contiguous pumps form a pipe.
                let (dx, dy) =
                    [(1, 0), (0, 1), (-1, 0), (0, -1)][(self.cells[i].life % 4) as usize];
                let source = self.at(x - dx, y - dy);
                if matches!(
                    source,
                    Water | Oil | Soapy | SaltWater | Mercury | Nitro | Acid | Gas | Steam
                ) {
                    let (mut tx, mut ty) = (x + dx, y + dy);
                    while self.at(tx, ty) == Pump {
                        tx += dx;
                        ty += dy;
                    }
                    if self.at(tx, ty) == Empty {
                        let from = self.index(x - dx, y - dy).unwrap();
                        let to = self.index(tx, ty).unwrap();
                        self.cells[to] = self.cells[from];
                        self.motion[to] = self.motion[from];
                        self.visited[to] = self.tick;
                        self.set(x - dx, y - dy, Empty);
                    }
                }
            }
            Glass => {
                let (_, _, p) = self.air.sample(x as f32, y as f32);
                if p.abs() > 2.5 {
                    self.set(x, y, Sand);
                }
            }
            Vine => {
                if hot {
                    self.set(x, y, Fire);
                } else if self.random() % 80 == 0 {
                    let watered = ns
                        .iter()
                        .any(|&(a, b)| matches!(self.at(a, b), Water | SaltWater));
                    if watered {
                        let nx = x + dir;
                        let ny = y - 1;
                        if self.at(nx, ny) == Empty {
                            self.set(nx, ny, Vine);
                        }
                    }
                }
            }
            Bird => {
                if hot {
                    self.set(x, y, Fire);
                } else {
                    let mut flock = 0;
                    for offset in [-6, -3, 3, 6] {
                        if self.at(x + offset, y) == Bird {
                            flock += offset.signum();
                        }
                    }
                    let (wx, wy, _) = self.air.sample(x as f32, y as f32);
                    let dx = if flock != 0 {
                        flock.signum()
                    } else if self.cells[i].life & 1 == 0 {
                        1
                    } else {
                        -1
                    };
                    let dy = if wy.abs() > 0.5 {
                        wy.signum() as i32
                    } else if self.random() % 5 == 0 {
                        dir
                    } else {
                        0
                    };
                    if !self.exchange(
                        i,
                        x + (dx + (wx * 0.2).round() as i32).clamp(-1, 1),
                        y + dy,
                        true,
                    ) {
                        self.cells[i].life ^= 1;
                    }
                }
            }
            Cloud => {
                if self.random() % 100 == 0 && self.at(x, y + 1) == Empty {
                    let drop = if ns.iter().any(|&(a, b)| matches!(self.at(a, b), Ice | Snow)) {
                        Snow
                    } else if hot {
                        Thunder
                    } else {
                        Water
                    };
                    self.set(x, y + 1, drop);
                }
                let (vx, _, _) = self.air.sample(x as f32, y as f32);
                if vx.abs() > 0.3 {
                    self.exchange(i, x + vx.signum() as i32, y, true);
                }
            }
            Ant => {
                if hot {
                    self.set(x, y, Fire);
                } else {
                    let heading = self.cells[i].life % 4;
                    let (dx, dy) = [(1, 0), (0, -1), (-1, 0), (0, 1)][heading as usize];
                    let target = self.at(x + dx, y + dy);
                    if matches!(target, Wood | Seed | Vine | Sand) {
                        self.set(x + dx, y + dy, Empty);
                        self.exchange(i, x + dx, y + dy, true);
                    } else if target == Empty {
                        if self.at(x, y + 1) == Empty {
                            self.exchange(i, x, y + 1, true);
                        } else {
                            self.exchange(i, x + dx, y + dy, true);
                        }
                    } else {
                        self.cells[i].life = (heading + 1) % 4;
                    }
                }
            }
            Virus => {
                if self.cells[i].life == 0 || hot {
                    self.set(x, y, Empty);
                } else {
                    self.cells[i].life -= 1;
                    for &(a, b) in ns {
                        let other = self.at(a, b);
                        if !matches!(other, Empty | Wall | Clone | Virus | Fan | Pump)
                            && self.random() % 5 == 0
                        {
                            self.set(a, b, Virus);
                            if let Some(j) = self.index(a, b) {
                                self.cells[j].life = self.cells[i].life;
                            }
                        }
                    }
                    self.exchange(i, x, y + 1, false);
                }
            }
            Firework => {
                if self.cells[i].life >> 6 == 0 {
                    if hot {
                        self.cells[i].life |= 64;
                    } else {
                        self.exchange(i, x, y + 1, false);
                    }
                } else if self.cells[i].life >> 6 == 3 {
                    let payload = Material::from_u8(self.cells[i].life & 63).unwrap_or(Sand);
                    self.set(x, y, Empty);
                    self.air.add_pressure(x as f32, y as f32, 3., 10.);
                    if payload.is_cell() && !matches!(payload, Empty | Firework | Clone) {
                        for n in 0..24 {
                            let a = n as f32 * std::f32::consts::TAU / 24.;
                            let px = x + (a.cos() * 7.).round() as i32;
                            let py = y + (a.sin() * 7.).round() as i32;
                            if self.at(px, py) == Empty {
                                self.set(px, py, payload);
                                if let Some(j) = self.index(px, py) {
                                    self.motion[j] = Motion {
                                        vx: a.cos() * 5.,
                                        vy: a.sin() * 5.,
                                    };
                                }
                            }
                        }
                    }
                } else {
                    if self.tick % 10 == 0 {
                        self.cells[i].life = self.cells[i].life.saturating_add(64);
                    }
                    self.exchange(i, x + dir, y - 1, true);
                    if self.at(x, y + 1) == Empty {
                        self.set(x, y + 1, Fire);
                    }
                }
            }
            Superball => {
                let up = self.cells[i].life > 0;
                if up {
                    self.cells[i].life -= 1;
                    if !self.exchange(i, x + dir, y - 1, true) {
                        self.cells[i].life = 0;
                    }
                } else if !self.exchange(i, x, y + 1, false) {
                    self.cells[i].life = 12;
                    self.exchange(i, x + dir, y, true);
                }
            }
            Bomb => {
                if self.cells[i].life & 128 != 0 {
                    if self.random() % 20 == 0 {
                        let original = Material::from_u8(self.cells[i].life & 127).unwrap_or(Empty);
                        let restored = match original {
                            Gunpowder | Nitro | Gas | Oil | Seed | Wood => Fire,
                            Water | Ice | Snow => Steam,
                            other => other,
                        };
                        self.set(x, y, restored);
                    }
                } else if hot || ns.iter().any(|&(a, b)| self.at(a, b) == Sand) {
                    self.bomb_collapse(x, y);
                } else {
                    self.exchange(i, x, y + 1, false);
                }
            }
            Nitro => {
                let (_, _, pressure) = self.air.sample(x as f32, y as f32);
                if hot || pressure.abs() > 2. {
                    self.explode(x, y, 10);
                } else {
                    self.fall_liquid(i, x, y, dir);
                }
            }
            Soapy => {
                if hot {
                    self.set(x, y, Steam);
                } else {
                    let (vx, vy, p) = self.air.sample(x as f32, y as f32);
                    if vx.abs() + vy.abs() + p.abs() > 0.8
                        && self.random() % 16 == 0
                        && self.bodies.len() < MAX_BODIES
                    {
                        self.bodies
                            .push(Body::new(BodyKind::Bubble, x as f32, y as f32 - 3., 3.));
                        self.set(x, y, Empty);
                    } else {
                        self.fall_liquid(i, x, y, dir);
                    }
                }
            }
            Salt => {
                let water = ns.iter().copied().find(|&(a, b)| self.at(a, b) == Water);
                if let Some((a, b)) = water {
                    self.set(a, b, SaltWater);
                    self.set(x, y, Empty);
                } else if !self.exchange(i, x, y + 1, false) {
                    self.exchange(i, x + dir, y + 1, false);
                }
            }
            SaltWater => {
                if hot {
                    self.set(x, y, Salt);
                    for &(a, b) in ns {
                        if self.at(a, b) == Empty {
                            self.set(a, b, Steam);
                            break;
                        }
                    }
                } else {
                    self.fall_liquid(i, x, y, dir);
                }
            }
            Mercury => {
                self.fall_liquid(i, x, y, dir);
            }
            Snow => {
                if hot
                    || ns
                        .iter()
                        .any(|&(a, b)| matches!(self.at(a, b), Water | SaltWater))
                {
                    self.set(x, y, Water);
                } else if !self.exchange(i, x, y + 1, false) {
                    self.exchange(i, x + dir, y + 1, false);
                }
            }
            _ => return false,
        }
        true
    }

    fn fall_liquid(&mut self, i: usize, x: i32, y: i32, dir: i32) {
        if self.fast_paths() && x > 0 && x + 1 < self.width as i32 && y + 1 < self.height as i32 {
            // These neighbors are already in bounds. Preserve exactly the same order
            // of density exchanges without recomputing four x/y grid addresses.
            let below = i + self.width;
            let material = self.cells[i].material;
            let density = material.density();
            for j in [
                below,
                (below as isize + dir as isize) as usize,
                (below as isize - dir as isize) as usize,
                (i as isize + dir as isize) as usize,
            ] {
                let other = self.cells[j].material;
                if other != material && density > other.density() {
                    self.cells.swap(i, j);
                    self.motion.swap(i, j);
                    self.visited[i] = self.tick;
                    self.visited[j] = self.tick;
                    return;
                }
            }
        } else {
            if self.exchange(i, x, y + 1, false)
                || self.exchange(i, x + dir, y + 1, false)
                || self.exchange(i, x - dir, y + 1, false)
            {
                return;
            }
            self.exchange(i, x + dir, y, false);
        }
    }
    fn step_actors(&mut self, input: Input) {
        actors::step(
            &mut self.actors,
            &self.cells,
            self.width,
            self.height,
            &self.air,
            input,
            self.tick,
        );
    }
    /// Copy only presentation data; simulation buffers remain owned by the worker.
    pub fn copy_visual_from(&mut self, other: &Self) {
        assert_eq!((self.width, self.height), (other.width, other.height));
        self.cells.copy_from_slice(&other.cells);
        self.air.nodes.copy_from_slice(&other.air.nodes);
        self.actors.clone_from(&other.actors);
        self.bodies.clone_from(&other.bodies);
        self.brush_payload = other.brush_payload;
        self.tick = other.tick;
    }
    pub fn render(&self, pixels: &mut [u32]) {
        assert_eq!(pixels.len(), self.cells.len());
        for (i, (pixel, cell)) in pixels.iter_mut().zip(&self.cells).enumerate() {
            let base = cell.material.color();
            *pixel = if cell.material == Material::Fire {
                if (cell.life as usize + i) % 3 == 0 {
                    0xffda48
                } else {
                    base
                }
            } else {
                base
            };
        }
        render_bodies(&self.bodies, pixels, self.width, self.height);
        actors::render(&self.actors, pixels, self.width, self.height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dense_fast_paths_preserve_complete_state_through_wind_blasts_and_edits() {
        for scene in 0..3 {
            let make = || {
                let mut w = World::new(80, 64);
                w.cells.fill(Cell::new(Material::Water));
                if scene >= 1 {
                    for y in 20..30 {
                        for x in 30..40 {
                            w.cells[y * 80 + x] = Cell::new(Material::Nitro);
                        }
                    }
                    w.apply_pressure(35, 25, 10, 10.);
                }
                if scene == 2 {
                    for x in 8..70 {
                        w.cells[50 * 80 + x] = Cell::new(Material::Wall);
                    }
                    for (x, m) in [
                        (10, Material::Salt),
                        (15, Material::Magma),
                        (20, Material::Sand),
                        (25, Material::Oil),
                        (45, Material::Fire),
                    ] {
                        w.cells[10 * 80 + x] = Cell::new(m);
                    }
                    w.bodies.push(Body::new(BodyKind::Wheel, 65., 40., 5.));
                    w.actors.push(Actor::new(65., 20., Material::Player));
                }
                w.motion[3] = Motion { vx: -0., vy: -0. };
                w
            };
            let mut optimized = make();
            let mut reference = make();
            reference.reference_steps = true;
            for tick in 0..160 {
                for w in [&mut optimized, &mut reference] {
                    if tick == 30 {
                        w.set(40, 40, Material::Fan);
                    }
                    if tick == 60 {
                        w.paint_line(10, 35, 65, 35, 0, Material::Empty);
                    }
                    if tick == 90 {
                        w.air.add_wind(60., 15., -4., 2., 20.);
                    }
                    if tick == 120 {
                        w.drag_at(50, 25, 3, -2, 6);
                    }
                }
                let input = Input {
                    left: tick % 30 < 10,
                    right: tick % 30 >= 20,
                    jump: tick % 40 == 0,
                };
                optimized.step(input);
                reference.step(input);
                assert_eq!(
                    optimized.cells, reference.cells,
                    "scene={scene} tick={tick} cells"
                );
                assert_eq!(
                    optimized.motion, reference.motion,
                    "scene={scene} tick={tick} momentum"
                );
                assert_eq!(
                    optimized.air.nodes, reference.air.nodes,
                    "scene={scene} tick={tick} wind"
                );
                assert_eq!(
                    optimized.rng, reference.rng,
                    "scene={scene} tick={tick} random state"
                );
            }
            let p =
                std::env::temp_dir().join(format!("powder-exact-{}-{scene}", std::process::id()));
            let q = p.with_extension("reference");
            optimized.save(&p).unwrap();
            reference.save(&q).unwrap();
            assert_eq!(
                std::fs::read(&p).unwrap(),
                std::fs::read(&q).unwrap(),
                "complete saved state scene={scene}"
            );
            std::fs::remove_file(p).unwrap();
            std::fs::remove_file(q).unwrap();
        }
    }
    #[test]
    fn old_saves_and_updates_clear_subnormal_air_and_momentum() {
        let mut w = World::new(32, 32);
        w.cells.fill(Cell::new(Material::Water));
        for node in &mut w.air.nodes {
            node.vx = f32::from_bits(1);
            node.vy = -f32::from_bits(500);
            node.pressure = f32::from_bits(900);
        }
        for motion in &mut w.motion {
            motion.vx = -f32::from_bits(12);
            motion.vy = f32::from_bits(300);
        }
        let path = std::env::temp_dir().join(format!("powder-subnormal-{}", std::process::id()));
        w.save(&path).unwrap();
        let loaded = World::load(&path).unwrap();
        assert!(loaded
            .air
            .nodes
            .iter()
            .all(|n| n.vx == 0. && n.vy == 0. && n.pressure == 0.));
        assert!(loaded.motion.iter().all(|m| *m == Motion::default()));
        w.step(Input::default());
        assert!(w
            .air
            .nodes
            .iter()
            .all(|n| n.vx == 0. && n.vy == 0. && n.pressure == 0.));
        assert!(w.motion.iter().all(|m| *m == Motion::default()));
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn uniform_passive_regions_match_reference_with_wind_heat_and_conductors() {
        for material in [
            Material::Water,
            Material::Oil,
            Material::Sand,
            Material::Gunpowder,
            Material::Stone,
            Material::Seed,
            Material::Salt,
            Material::SaltWater,
            Material::Mercury,
        ] {
            let mut a = World::new(32, 32);
            let mut b = World::new(32, 32);
            a.cells.fill(Cell::new(material));
            b.cells.clone_from(&a.cells);
            b.reference_steps = true;
            for tick in 0..64 {
                for w in [&mut a, &mut b] {
                    if tick == 10 {
                        w.air.add_wind(16., 16., 5., -2., 16.);
                    }
                    if tick == 20 && matches!(material, Material::SaltWater | Material::Mercury) {
                        w.cells[16 * 32 + 16].life = 136;
                    }
                    if tick == 30 {
                        w.set(16, 17, Material::Fire);
                    }
                    if tick == 40 {
                        w.paint_line(8, 14, 23, 14, 0, Material::Empty);
                    }
                }
                a.step(Input::default());
                b.step(Input::default());
                assert_eq!(a.cells, b.cells, "{material:?} tick={tick}");
                assert_eq!(a.motion, b.motion, "{material:?} tick={tick}");
                assert_eq!(a.air.nodes, b.air.nodes, "{material:?} tick={tick}");
                assert_eq!(a.rng, b.rng, "{material:?} tick={tick}");
            }
        }
    }
    #[test]
    fn velocity_length_shortcut_is_bit_exact_for_tiny_and_large_components() {
        let mut a = World::new(16, 16);
        let mut b = World::new(16, 16);
        b.reference_steps = true;
        let values = [
            0., -0., 1e-40, -1e-40, 1e-30, -1e-30, 1e-20, -1e-20, 1e-10, -1e-10, 0.001, -0.001, 1.,
            -1., 2., -2., 3.5, -3.5, 3.8, -3.8, 12., -12., 24., -24.,
        ];
        for x in values {
            for y in values {
                a.motion[17] = Motion { vx: x, vy: y };
                b.motion[17] = a.motion[17];
                a.rng = 0x123456789abcdef;
                b.rng = a.rng;
                let signature = |step: Option<MotionStep>| {
                    step.map(|s| {
                        (
                            s.vx.to_bits(),
                            s.vy.to_bits(),
                            s.steps,
                            s.fraction.to_bits(),
                        )
                    })
                };
                assert_eq!(
                    signature(a.integrate_motion(17, 1, 1, 0.075)),
                    signature(b.integrate_motion(17, 1, 1, 0.075)),
                    "vx={x} vy={y}"
                );
                assert_eq!(a.rng, b.rng);
            }
        }
    }
    #[test]
    fn enclosed_particle_shortcut_matches_swept_collisions() {
        let mut a = World::new(32, 32);
        let mut b = World::new(32, 32);
        a.cells.fill(Cell::new(Material::Water));
        b.cells.clone_from(&a.cells);
        b.reference_steps = true;
        let i = 16 * 32 + 16;
        for x in -48..=48 {
            for y in -48..=48 {
                let motion = Motion {
                    vx: x as f32 * 0.5,
                    vy: y as f32 * 0.5,
                };
                a.motion[i] = motion;
                b.motion[i] = motion;
                a.rng = (x + 50) as u64 * 65537 + (y + 50) as u64;
                b.rng = a.rng;
                assert_eq!(
                    a.transport_particle(i, 16, 16, Material::Water),
                    b.transport_particle(i, 16, 16, Material::Water)
                );
                assert_eq!(a.motion[i], b.motion[i], "vx={x} vy={y}");
                assert_eq!(a.rng, b.rng);
            }
        }
    }
    #[test]
    fn grid_index_rejects_negative_and_extreme_coordinates() {
        let w = World::new(16, 18);
        for y in [i32::MIN, -1, 0, 15, 16, 17, 18, i32::MAX] {
            for x in [i32::MIN, -1, 0, 15, 16, 17, 18, i32::MAX] {
                let expected = if (0..16).contains(&x) && (0..18).contains(&y) {
                    Some(y as usize * 16 + x as usize)
                } else {
                    None
                };
                assert_eq!(w.index(x, y), expected);
            }
        }
    }
    #[test]
    fn chain_reactions_have_a_per_update_budget_and_resume_after_saving() {
        let mut world = World::new(96, 64);
        for y in 0..5 {
            for x in 0..20 {
                let i = (10 + y * 3) * 96 + 10 + x * 3;
                world.cells[i] = Cell {
                    material: Material::C4,
                    life: 1,
                };
            }
        }
        world.step(Input::default());
        assert_eq!(world.blasts_this_step, MAX_BLASTS_PER_STEP);
        assert_eq!(
            world
                .cells
                .iter()
                .filter(|c| c.material == Material::C4)
                .count(),
            100 - MAX_BLASTS_PER_STEP
        );
        let path =
            std::env::temp_dir().join(format!("surface-pending-blasts-{}.sav", std::process::id()));
        world.save(&path).unwrap();
        let mut loaded = World::load(&path).unwrap();
        loaded.step(Input::default());
        assert!(!loaded.cells.iter().any(|c| c.material == Material::C4));
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn deferred_explosives_detonate_without_remaining_next_to_fire() {
        let mut world = World::new(96, 64);
        world.blasts_this_step = MAX_BLASTS_PER_STEP;
        for (x, material) in [
            (10, Material::Gunpowder),
            (30, Material::Gas),
            (50, Material::Nitro),
            (70, Material::C4),
        ] {
            world.set(x, 20, material);
            world.explode(x, 20, 10);
            assert_eq!(world.cells[20 * 96 + x as usize].life, 1);
        }
        world.step(Input::default());
        assert_eq!(world.blasts_this_step, 4);
        assert!(!world.cells.iter().any(|c| matches!(
            c.material,
            Material::Gunpowder | Material::Gas | Material::Nitro | Material::C4
        )));
    }
    #[test]
    fn blast_preserves_powder_and_gives_outward_momentum() {
        let mut w = World::new(64, 64);
        w.set(37, 32, Material::Sand);
        w.set(27, 32, Material::Water);
        w.explode(32, 32, 10);
        assert_eq!(w.at(37, 32), Material::Sand);
        assert_eq!(w.at(27, 32), Material::Water);
        assert!(w.motion[32 * 64 + 37].vx > 3.);
        assert!(w.motion[32 * 64 + 27].vx < -3.);
        for _ in 0..4 {
            w.step(Input::default());
        }
        assert!(w
            .cells
            .iter()
            .enumerate()
            .any(|(i, c)| c.material == Material::Sand && i % 64 > 40));
    }
    #[test]
    fn blast_cannot_kick_particles_through_a_wall() {
        let mut w = World::new(64, 64);
        w.paint_line(34, 0, 34, 63, 0, Material::Wall);
        w.set(37, 32, Material::Sand);
        w.explode(32, 32, 10);
        assert_eq!(w.motion[32 * 64 + 37], Motion::default());
        w.motion[32 * 64 + 37] = Motion { vx: -20., vy: 0. };
        w.step(Input::default());
        assert!(!w
            .cells
            .iter()
            .enumerate()
            .any(|(i, c)| c.material == Material::Sand && i % 64 < 34));
    }
    #[test]
    fn v2_saves_upgrade_and_v3_retains_particle_inertia() {
        let mut w = World::new(16, 16);
        let path =
            std::env::temp_dir().join(format!("surface-momentum-{}.sav", std::process::id()));
        w.set(8, 8, Material::Sand);
        w.motion[8 * 16 + 8] = Motion { vx: 3.5, vy: -2. };
        w.save(&path).unwrap();
        let loaded = World::load(&path).unwrap();
        assert_eq!(loaded.motion, w.motion);
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[8] = 2;
        bytes.truncate(bytes.len() - w.cells.len() * 8);
        std::fs::write(&path, &bytes).unwrap();
        let legacy = World::load(&path).unwrap();
        assert!(legacy.motion.iter().all(|m| *m == Motion::default()));
        legacy.save(&path).unwrap();
        let backup = format!("{}.v2.bak", path.display());
        assert_eq!(std::fs::read(&backup).unwrap(), bytes);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(backup).unwrap();
    }
    #[test]
    fn compact_cells() {
        assert_eq!(std::mem::size_of::<Cell>(), 2);
    }
    #[test]
    fn powder_falls_and_is_conserved() {
        let mut w = World::new(32, 32);
        w.paint_line(8, 2, 23, 2, 1, Material::Sand);
        let count = w
            .cells
            .iter()
            .filter(|c| c.material == Material::Sand)
            .count();
        for _ in 0..60 {
            w.step(Input::default());
        }
        assert_eq!(
            w.cells
                .iter()
                .filter(|c| c.material == Material::Sand)
                .count(),
            count
        );
        assert!(!w.cells[..16 * 32]
            .iter()
            .any(|c| c.material == Material::Sand));
    }
    #[test]
    fn water_does_not_pass_wall() {
        let mut w = World::new(32, 32);
        w.paint_line(0, 20, 31, 20, 0, Material::Wall);
        w.paint_line(16, 4, 16, 4, 3, Material::Water);
        for _ in 0..60 {
            w.step(Input::default());
        }
        assert!(!w.cells[21 * 32..]
            .iter()
            .any(|c| c.material == Material::Water));
    }
    #[test]
    fn magma_and_water_make_steam_and_stone() {
        let mut w = World::new(32, 32);
        w.set(15, 15, Material::Magma);
        w.set(15, 14, Material::Water);
        w.paint_line(14, 16, 17, 16, 0, Material::Wall);
        for _ in 0..3 {
            w.step(Input::default());
        }
        assert!(w.cells.iter().any(|c| c.material == Material::Steam));
        assert!(w.cells.iter().any(|c| c.material == Material::Stone));
    }
    #[test]
    fn player_moves_and_jumps() {
        let mut w = World::new(64, 64);
        w.paint_line(0, 50, 63, 50, 0, Material::Wall);
        w.paint_line(20, 42, 20, 42, 0, Material::Player);
        for _ in 0..30 {
            w.step(Input {
                right: true,
                ..Input::default()
            });
        }
        assert!(w.actors[0].x > 22., "{:?}", w.actors[0]);
        let y = w.actors[0].y;
        for _ in 0..10 {
            w.step(Input {
                jump: true,
                ..Input::default()
            });
        }
        assert!(w.actors[0].y < y);
    }
    #[test]
    fn clone_replicates_sample() {
        let mut w = World::new(32, 32);
        w.set(10, 10, Material::Clone);
        w.set(10, 11, Material::Wall);
        w.set(9, 10, Material::Wall);
        w.set(11, 10, Material::Wall);
        w.set(10, 9, Material::Sand);
        for _ in 0..30 {
            w.step(Input::default());
        }
        assert_eq!(w.cells[10 * 32 + 10].life, Material::Sand as u8);
    }
    #[test]
    fn player_lands_on_single_pixel_platform() {
        let mut w = World::new(64, 64);
        w.paint_line(0, 50, 63, 50, 0, Material::Wall);
        w.paint_line(20, 10, 20, 10, 0, Material::Player);
        for _ in 0..40 {
            w.step(Input::default());
        }
        assert!(w.actors[0].joints[4..6]
            .iter()
            .all(|j| j.y < 50. && j.y > 47.));
        assert!(w.actors[0].y < 48.);
    }
    #[test]
    fn conductive_spark_preserves_the_metal_cell() {
        let mut w = World::new(32, 32);
        w.set(10, 10, Material::Spark);
        w.cells[10 * 32 + 10].life = 128 | 8;
        for _ in 0..8 {
            w.step(Input::default());
        }
        assert_eq!(w.at(10, 10), Material::Metal);
    }
    #[test]
    fn save_roundtrip_retains_simulation_and_actors() {
        let mut w = World::new(64, 64);
        w.seed_demo();
        for _ in 0..4 {
            w.step(Input::default());
        }
        let path =
            std::env::temp_dir().join(format!("surface-dust-test-{}.sav", std::process::id()));
        w.save(&path).unwrap();
        let mut restored = World::load(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(w.cells, restored.cells);
        assert_eq!(w.actors.len(), restored.actors.len());
        assert_eq!(w.actors[0].x, restored.actors[0].x);
        w.step(Input::default());
        restored.step(Input::default());
        assert_eq!(w.cells, restored.cells);
    }
    fn react(w: &mut World, x: i32, y: i32) {
        let i = w.index(x, y).unwrap();
        let m = w.cells[i].material;
        w.step_new_material(
            i,
            x,
            y,
            m,
            1,
            &[(x, y - 1), (x + 1, y), (x - 1, y), (x, y + 1)],
        );
    }
    #[test]
    fn new_material_ids_remain_compatible_and_tools_never_become_cells() {
        assert_eq!(Material::Fighter as u8, 19);
        assert_eq!(Material::ALL.len(), 50);
        let mut w = World::new(64, 64);
        for m in [
            Material::Wind,
            Material::Air,
            Material::Drag,
            Material::Wheel,
            Material::Box,
            Material::Bubble,
            Material::Ball,
            Material::Create,
        ] {
            w.paint_line(20, 20, 21, 20, 3, m);
        }
        assert!(w.cells.iter().all(|c| c.material.is_cell()));
        assert!(!w.bodies.is_empty());
    }
    #[test]
    fn salt_dissolves_and_heating_brine_recovers_salt() {
        let mut w = World::new(32, 32);
        w.set(10, 10, Material::Salt);
        w.set(10, 11, Material::Water);
        react(&mut w, 10, 10);
        assert_eq!(w.at(10, 11), Material::SaltWater);
        assert_eq!(w.at(10, 10), Material::Empty);
        w.set(11, 11, Material::Torch);
        react(&mut w, 10, 11);
        assert_eq!(w.at(10, 11), Material::Salt);
        assert!(w.cells.iter().any(|c| c.material == Material::Steam));
    }
    #[test]
    fn fireworks_capture_payload_and_fuse_burns_slowly() {
        let mut w = World::new(64, 64);
        w.brush_payload = Material::Water;
        w.paint_line(30, 30, 30, 30, 0, Material::Firework);
        w.brush_payload = Material::Sand;
        w.ignite(30, 30);
        assert_eq!(w.at(30, 30), Material::Firework);
        let i = w.index(30, 30).unwrap();
        w.cells[i].life |= 192;
        react(&mut w, 30, 30);
        assert!(w.cells.iter().any(|c| c.material == Material::Water));
        w.set(10, 10, Material::Fuse);
        w.set(11, 10, Material::Fuse);
        w.ignite(10, 10);
        assert_eq!(w.at(10, 10), Material::Fuse);
        for _ in 0..18 {
            react(&mut w, 10, 10);
        }
        assert_eq!(w.at(10, 10), Material::Fire);
        assert!(w.cells[w.index(11, 10).unwrap()].life > 0);
    }
    #[test]
    fn laser_reflects_without_destroying_wall_and_pump_conserves_state() {
        let mut w = World::new(32, 32);
        w.set(10, 10, Material::Laser);
        w.set(11, 10, Material::Metal);
        react(&mut w, 10, 10);
        assert_eq!(w.at(9, 10), Material::Laser);
        assert_eq!(w.cells[w.index(9, 10).unwrap()].life >> 6, 2);
        w.set(10, 10, Material::Laser);
        w.set(9, 10, Material::Wall);
        react(&mut w, 10, 10);
        assert_eq!(w.at(9, 10), Material::Wall);
        assert_eq!(w.at(11, 10), Material::Metal);
        w.set(20, 20, Material::Pump);
        w.set(20, 19, Material::Pump);
        let pump = w.index(20, 20).unwrap();
        w.cells[pump].life = 3;
        w.set(20, 21, Material::Steam);
        let i = w.index(20, 21).unwrap();
        w.cells[i].life = 31;
        react(&mut w, 20, 20);
        assert_eq!(w.at(20, 21), Material::Empty);
        assert_eq!(w.at(20, 18), Material::Steam);
        assert_eq!(w.cells[w.index(20, 18).unwrap()].life, 31);
    }
    #[test]
    fn bombs_respond_to_powder_and_pressure_shatters_glass() {
        let mut w = World::new(64, 64);
        w.set(20, 20, Material::Bomb);
        w.set(21, 20, Material::Sand);
        react(&mut w, 20, 20);
        assert_eq!(w.at(20, 20), Material::Empty);
        assert_eq!(w.at(21, 20), Material::Bomb);
        assert!(w.motion[20 * 64 + 21].vx < 0.);
        w.set(45, 45, Material::Glass);
        w.apply_pressure(45, 45, 8, 10.);
        react(&mut w, 45, 45);
        assert_eq!(w.at(45, 45), Material::Sand);
    }
    #[test]
    fn life_and_weather_have_distinct_behaviors() {
        let mut w = World::new(64, 64);
        w.set(10, 10, Material::Ant);
        w.set(11, 10, Material::Wood);
        react(&mut w, 10, 10);
        assert_eq!(w.at(11, 10), Material::Ant);
        w.set(20, 20, Material::Torch);
        w.set(21, 20, Material::Water);
        react(&mut w, 20, 20);
        assert_eq!(w.at(20, 20), Material::Wood);
        w.set(30, 30, Material::Snow);
        w.set(31, 30, Material::Fire);
        react(&mut w, 30, 30);
        assert_eq!(w.at(30, 30), Material::Water);
        w.set(40, 40, Material::Cloud);
        for _ in 0..1000 {
            react(&mut w, 40, 40);
        }
        assert_eq!(w.at(40, 41), Material::Water);
        w.set(50, 40, Material::Virus);
        w.set(51, 40, Material::Stone);
        w.set(50, 41, Material::Wall);
        for _ in 0..30 {
            react(&mut w, 50, 40);
        }
        assert_eq!(w.at(51, 40), Material::Virus);
    }
    #[test]
    fn save_v2_continues_air_bodies_and_payload_deterministically() {
        let mut w = World::new(64, 64);
        w.brush_payload = Material::Salt;
        w.paint_line(10, 10, 10, 10, 4, Material::Box);
        w.paint_line(50, 10, 50, 10, 3, Material::Ball);
        w.paint_line(30, 30, 35, 30, 8, Material::Wind);
        w.paint_line(30, 20, 30, 20, 4, Material::Snow);
        for _ in 0..5 {
            w.step(Input::default());
        }
        let path = std::env::temp_dir().join(format!("surface-v2-{}.sav", std::process::id()));
        w.save(&path).unwrap();
        let mut loaded = World::load(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        for _ in 0..20 {
            w.step(Input::default());
            loaded.step(Input::default());
        }
        assert_eq!(w.cells, loaded.cells);
        assert_eq!(w.air.nodes, loaded.air.nodes);
        assert_eq!(w.bodies[0].x, loaded.bodies[0].x);
        assert_eq!(loaded.brush_payload, Material::Salt);
        assert_eq!(loaded.bodies[1].payload, Material::Salt);
    }
    #[test]
    fn legacy_save_migrates_and_is_backed_up() {
        let w = World::new(16, 16);
        let path = std::env::temp_dir().join(format!("surface-v1-{}.sav", std::process::id()));
        w.save(&path).unwrap();
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[8] = 1;
        bytes.truncate(29 + 16 * 16 * 2 + 1);
        std::fs::write(&path, &bytes).unwrap();
        let loaded = World::load(&path).unwrap();
        assert!(loaded.bodies.is_empty());
        loaded.save(&path).unwrap();
        let backup = format!("{}.v1.bak", path.display());
        assert_eq!(std::fs::read(&backup).unwrap(), bytes);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(backup).unwrap();
    }
    #[test]
    fn saves_reject_tool_cells_and_nonfinite_air() {
        let w = World::new(16, 16);
        let path = std::env::temp_dir().join(format!("surface-invalid-{}.sav", std::process::id()));
        w.save(&path).unwrap();
        let original = std::fs::read(&path).unwrap();
        let mut bytes = original.clone();
        bytes[29] = Material::Wind as u8;
        std::fs::write(&path, &bytes).unwrap();
        assert!(World::load(&path).is_err());
        let mut bytes = original;
        let first_air = 29 + 16 * 16 * 2 + 1;
        bytes[first_air..first_air + 4].copy_from_slice(&f32::NAN.to_le_bytes());
        std::fs::write(&path, &bytes).unwrap();
        assert!(World::load(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn conducting_metal_stays_put_in_wind() {
        let mut w = World::new(32, 32);
        w.set(10, 10, Material::Spark);
        let i = w.index(10, 10).unwrap();
        w.cells[i].life = 128 | 8;
        w.air.add_wind(10., 10., 20., 0., 12.);
        for _ in 0..8 {
            w.step(Input::default());
        }
        assert_eq!(w.at(10, 10), Material::Metal);
    }
    #[test]
    fn fan_and_pump_follow_stroke_direction() {
        let mut w = World::new(64, 64);
        w.paint_line(30, 30, 30, 28, 0, Material::Fan);
        react(&mut w, 30, 28);
        assert!(w.air.sample(30., 28.).1 < 0.);
        w.paint_line(20, 20, 19, 20, 0, Material::Pump);
        w.set(21, 20, Material::Water);
        react(&mut w, 20, 20);
        assert_eq!(w.at(18, 20), Material::Water);
        assert_eq!(w.at(21, 20), Material::Empty);
    }
    #[test]
    fn dragging_wheel_rim_spins_without_moving_axle() {
        let mut w = World::new(64, 64);
        w.paint_line(30, 30, 30, 30, 8, Material::Wheel);
        w.drag_at(38, 30, 0, 8, 2);
        assert!(w.bodies[0].angular_velocity > 0.);
        w.step(Input::default());
        assert!(w.bodies[0].angle > 0.);
        assert_eq!((w.bodies[0].x, w.bodies[0].y), (30., 30.));
    }
    #[test]
    fn wind_stops_at_one_pixel_barriers_and_boundary_strokes_work() {
        let mut w = World::new(32, 32);
        w.paint_line(16, 0, 16, 31, 0, Material::Wall);
        w.paint_line(14, 4, 14, 26, 0, Material::Water);
        for _ in 0..60 {
            w.air.add_wind(15., 16., 20., 0., 24.);
            w.step(Input::default());
        }
        assert!(!w
            .cells
            .iter()
            .enumerate()
            .any(|(i, c)| i % 32 > 16 && c.material == Material::Water));
        w.air.clear();
        w.paint_line(30, 16, 35, 16, 4, Material::Wind);
        assert!(w.air.sample(31., 16.).0 > 0.);
    }
}
