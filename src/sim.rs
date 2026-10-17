//! Original compact cellular sandbox. No upstream game code or assets are used.

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
}

impl Material {
    pub const ALL: [Self; 20] = [
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
        }
    }
    fn density(self) -> u8 {
        match self {
            Self::Empty => 0,
            Self::Fire | Self::Steam | Self::Gas | Self::Spark => 1,
            Self::Oil => 2,
            Self::Water | Self::Acid => 3,
            Self::Sand | Self::Seed | Self::Gunpowder => 5,
            Self::Stone => 6,
            Self::Magma => 4,
            _ => 255,
        }
    }
    fn combustible(self) -> bool {
        matches!(
            self,
            Self::Seed | Self::Wood | Self::Gunpowder | Self::Oil | Self::Gas
        )
    }
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

#[derive(Clone, Copy, Debug)]
pub struct Actor {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub kind: Material,
    pub health: u8,
}

pub struct World {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
    pub actors: Vec<Actor>,
    visited: Vec<u32>,
    tick: u32,
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
            visited: vec![0; width * height],
            tick: 0,
            rng: 0x69d2_739b_b853_a451,
        }
    }
    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
        self.actors.clear();
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
    }
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        use std::io::Write;
        let path = path.as_ref();
        let mut bytes = Vec::with_capacity(self.cells.len() * 2 + 64 + self.actors.len() * 18);
        bytes.extend_from_slice(b"SURFDUST\x01");
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
            .take(4_200_000)
            .read_to_end(&mut bytes)?;
        let mut cursor = 0;
        fn take<'a>(bytes: &'a [u8], cursor: &mut usize, n: usize) -> std::io::Result<&'a [u8]> {
            let part = bytes
                .get(*cursor..*cursor + n)
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Truncated sandbox save"))?;
            *cursor += n;
            Ok(part)
        }
        if take(&bytes, &mut cursor, 9)? != b"SURFDUST\x01" {
            return Err(invalid());
        }
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
            if matches!(material, Material::Player | Material::Fighter)
                || (material == Material::Clone && Material::from_u8(b[1]).is_none())
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
            world.actors.push(Actor {
                x: values[0],
                y: values[1],
                vx: values[2],
                vy: values[3],
                kind,
                health: b[1],
            });
        }
        if cursor != bytes.len() {
            return Err(invalid());
        }
        Ok(world)
    }
    fn random(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 16) as u32
    }
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        (x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height).then_some(
            (y.max(0) as usize)
                .saturating_mul(self.width)
                .saturating_add(x.max(0) as usize),
        )
    }
    pub fn at(&self, x: i32, y: i32) -> Material {
        self.index(x, y)
            .map(|i| self.cells[i].material)
            .unwrap_or(Material::Wall)
    }
    fn set(&mut self, x: i32, y: i32, material: Material) {
        if let Some(i) = self.index(x, y) {
            self.cells[i] = Cell::new(material);
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
        let radius = radius.clamp(0, 40);
        if matches!(material, Material::Player | Material::Fighter) {
            if self.index(x1, y1).is_some()
                && self.actors.len() < 64
                && !self
                    .actors
                    .iter()
                    .any(|a| (a.x - x1 as f32).abs() < 6. && (a.y - y1 as f32).abs() < 12.)
            {
                self.actors.push(Actor {
                    x: x1 as f32,
                    y: y1 as f32,
                    vx: 0.,
                    vy: 0.,
                    kind: material,
                    health: 255,
                });
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
                            }
                        }
                    }
                }
            }
        }
        if material == Material::Empty {
            self.actors.retain(|a| {
                (a.x - x1 as f32).abs() > radius as f32 + 3.
                    || (a.y - y1 as f32).abs() > radius as f32 + 8.
            });
        }
    }
    fn exchange(&mut self, i: usize, x: i32, y: i32, rising: bool) -> bool {
        let Some(j) = self.index(x, y) else {
            return false;
        };
        let a = self.cells[i].material;
        let b = self.cells[j].material;
        let allowed = if rising {
            b == Material::Empty
        } else {
            b.movable() && a.density() > b.density()
        };
        if allowed {
            self.cells.swap(i, j);
            self.visited[i] = self.tick;
            self.visited[j] = self.tick;
            true
        } else {
            false
        }
    }
    fn ignite(&mut self, x: i32, y: i32) {
        let m = self.at(x, y);
        if m.combustible() {
            if matches!(m, Material::Gunpowder | Material::Gas) {
                self.explode(x, y, if m == Material::Gas { 5 } else { 7 });
            } else {
                self.set(x, y, Material::Fire);
            }
        } else if m == Material::Ice {
            self.set(x, y, Material::Water);
        } else if m == Material::Water {
            self.set(x, y, Material::Steam);
        }
    }
    fn explode(&mut self, x: i32, y: i32, r: i32) {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let m = self.at(x + dx, y + dy);
                if !matches!(m, Material::Wall | Material::Metal | Material::Clone) {
                    self.set(x + dx, y + dy, Material::Fire);
                }
            }
        }
    }
    pub fn step(&mut self, input: Input) {
        self.tick = self.tick.wrapping_add(1);
        if self.tick == 0 {
            self.visited.fill(0);
            self.tick = 1;
        }
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
                let dir = if self.random() & 1 == 0 { -1 } else { 1 };
                let neighbors = [(x, y - 1), (x + dir, y), (x - dir, y), (x, y + 1)];
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
                                && self.at(nx, ny) == Material::Metal
                                && self.random() % 5 == 0
                            {
                                self.set(nx, ny, Material::Spark);
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
        self.step_actors(input);
    }
    fn fall_liquid(&mut self, i: usize, x: i32, y: i32, dir: i32) {
        if self.exchange(i, x, y + 1, false)
            || self.exchange(i, x + dir, y + 1, false)
            || self.exchange(i, x - dir, y + 1, false)
        {
            return;
        }
        self.exchange(i, x + dir, y, false);
    }
    fn actor_blocked(&self, x: f32, y: f32) -> bool {
        matches!(
            self.at(x as i32, y as i32),
            Material::Wall
                | Material::Wood
                | Material::Metal
                | Material::Clone
                | Material::Ice
                | Material::Stone
        )
    }
    fn step_actors(&mut self, input: Input) {
        for n in 0..self.actors.len() {
            let mut a = self.actors[n];
            let grounded =
                self.actor_blocked(a.x - 2., a.y + 8.) || self.actor_blocked(a.x + 2., a.y + 8.);
            let (direction, jump) = if a.kind == Material::Player {
                ((input.right as i8 - input.left as i8) as f32, input.jump)
            } else {
                let target = self
                    .actors
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != n)
                    .min_by(|(_, b), (_, c)| (b.x - a.x).abs().total_cmp(&(c.x - a.x).abs()))
                    .map(|(_, b)| b.x);
                let direction = target
                    .map(|x| if x < a.x { -1. } else { 1. })
                    .unwrap_or(if n % 2 == 0 { 1. } else { -1. });
                (
                    direction,
                    self.actor_blocked(a.x + direction * 4., a.y + 3.) || self.random() % 150 == 0,
                )
            };
            a.vx = (a.vx * 0.65 + direction * 0.5).clamp(-1.7, 1.7);
            if grounded {
                a.vy = 0.;
                if jump {
                    a.vy = -3.6;
                }
            }
            a.vy = (a.vy + 0.19).min(3.);
            // Sweep in subpixel steps so a thin drawn platform catches falling actors.
            let horizontal = a.vx.abs().ceil().max(1.) as usize;
            for _ in 0..horizontal {
                let nx = (a.x + a.vx / horizontal as f32).clamp(3., self.width as f32 - 4.);
                if !self.actor_blocked(nx, a.y) && !self.actor_blocked(nx, a.y + 6.) {
                    a.x = nx;
                } else {
                    a.vx = 0.;
                    break;
                }
            }
            let vertical = a.vy.abs().ceil().max(1.) as usize;
            for _ in 0..vertical {
                let ny = (a.y + a.vy / vertical as f32).clamp(5., self.height as f32 - 9.);
                if !self.actor_blocked(a.x, ny + 7.) && !self.actor_blocked(a.x, ny - 5.) {
                    a.y = ny;
                } else {
                    a.vy = 0.;
                    break;
                }
            }
            for (dx, dy) in [(0, 0), (0, 5), (-2, 7), (2, 7)] {
                if matches!(
                    self.at(a.x as i32 + dx, a.y as i32 + dy),
                    Material::Fire | Material::Magma | Material::Acid | Material::Spark
                ) {
                    a.health = a.health.saturating_sub(5);
                }
            }
            if a.kind == Material::Fighter {
                for j in 0..self.actors.len() {
                    if j != n
                        && (self.actors[j].x - a.x).abs() < 8.
                        && (self.actors[j].y - a.y).abs() < 10.
                    {
                        self.actors[j].vx = if self.actors[j].x < a.x { -3. } else { 3. };
                        self.actors[j].vy = -1.8;
                    }
                }
            }
            self.actors[n] = a;
        }
        self.actors.retain(|a| a.health > 0);
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
        for actor in &self.actors {
            let x = actor.x as i32;
            let y = actor.y as i32;
            let c = actor.kind.color();
            let stride = if actor.vx.abs() > 0.15 {
                ((self.tick / 5) % 2) as i32 * 2 - 1
            } else {
                0
            };
            let mut line = |x0: i32, y0: i32, x1: i32, y1: i32| {
                let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
                for n in 0..=steps {
                    if let Some(i) =
                        self.index(x0 + (x1 - x0) * n / steps, y0 + (y1 - y0) * n / steps)
                    {
                        pixels[i] = c;
                    }
                }
            };
            line(x - 2, y - 5, x + 2, y - 5);
            line(x - 2, y - 5, x - 2, y - 2);
            line(x + 2, y - 5, x + 2, y - 2);
            line(x - 2, y - 2, x + 2, y - 2);
            line(x, y - 1, x, y + 3);
            line(x, y, x - 4, y + 3);
            line(x, y, x + 4, y + 3);
            line(x, y + 3, x - 3 - stride, y + 7);
            line(x, y + 3, x + 3 + stride, y + 7);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        for _ in 0..5 {
            w.step(Input {
                right: true,
                ..Input::default()
            });
        }
        assert!(w.actors[0].x > 22.);
        let y = w.actors[0].y;
        w.step(Input {
            jump: true,
            ..Input::default()
        });
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
        assert!(w.actors[0].y < 43. && w.actors[0].y > 41.);
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
}
