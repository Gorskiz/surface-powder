//! Independent six-joint reconstruction of the original's loose, foot-driven characters.
use super::{Cell, Input, Material};
use crate::physics::Air;

#[derive(Clone, Copy, Debug, Default)]
pub struct Joint {
    pub x: f32,
    pub y: f32,
    pub old_x: f32,
    pub old_y: f32,
}
#[derive(Clone, Copy, Debug)]
pub struct Actor {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub kind: Material,
    pub health: u8,
    pub joints: [Joint; 11],
    pub dead: bool,
    pub age: u16,
    pub stride: u8,
    pub jump_wait: u8,
    pub knocked: u8,
}
impl Actor {
    pub fn new(x: f32, y: f32, kind: Material) -> Self {
        let mut joints = [Joint::default(); 11];
        for (joint, (dx, dy)) in joints.iter_mut().zip([
            (0., -4.),
            (0., 0.),
            (-2.5, 3.1),
            (2.5, 3.1),
            (-3., 8.),
            (3., 8.),
        ]) {
            *joint = Joint {
                x: x + dx,
                y: y + dy,
                old_x: x + dx,
                old_y: y + dy,
            };
        }
        Self {
            x,
            y,
            vx: 0.,
            vy: 0.,
            kind,
            health: 255,
            joints,
            dead: false,
            age: 0,
            stride: 0,
            jump_wait: 0,
            knocked: 0,
        }
    }
    pub fn impulse(&mut self, dx: f32, dy: f32) {
        for joint in &mut self.joints[..if self.dead { 11 } else { 6 }] {
            joint.old_x -= dx;
            joint.old_y -= dy;
        }
        self.vx += dx;
        self.vy += dy;
        self.knocked = 25;
        if self.kind == Material::Player && dx.abs() > 0.01 {
            self.health = 0;
        }
    }
    fn fragment(&mut self) {
        let old = self.joints;
        // Every limb becomes its own short thread; the head flies independently.
        for (i, source) in [0, 0, 1, 1, 1, 2, 2, 3, 3, 4, 5].into_iter().enumerate() {
            self.joints[i] = old[source];
        }
        for joint in &mut self.joints {
            joint.old_y += 1.;
        }
        self.dead = true;
        self.age = 0;
    }
}
fn solid(m: Material) -> bool {
    matches!(
        m,
        Material::Wall
            | Material::Wood
            | Material::Metal
            | Material::Clone
            | Material::Ice
            | Material::Stone
            | Material::Glass
            | Material::Fan
            | Material::Pump
            | Material::C4
    )
}
fn at(cells: &[Cell], w: usize, h: usize, x: f32, y: f32) -> Material {
    if x < 0. || y < 0. || x >= w as f32 || y >= h as f32 {
        Material::Wall
    } else {
        cells[y as usize * w + x as usize].material
    }
}
fn constrain(joints: &mut [Joint; 11], a: usize, b: usize, length: f32, stiffness: f32) {
    let dx = joints[b].x - joints[a].x;
    let dy = joints[b].y - joints[a].y;
    let distance = dx.hypot(dy);
    if distance > 0.0001 {
        let correction = (distance - length) / distance * 0.5 * stiffness;
        joints[a].x += dx * correction;
        joints[a].y += dy * correction;
        joints[b].x -= dx * correction;
        joints[b].y -= dy * correction;
    }
}
pub fn step(
    actors: &mut Vec<Actor>,
    cells: &[Cell],
    w: usize,
    h: usize,
    air: &Air,
    input: Input,
    tick: u32,
) {
    for (index, actor) in actors.iter_mut().enumerate() {
        if !actor.dead && actor.health == 0 {
            actor.fragment();
        }
        actor.age = actor.age.saturating_add(1);
        actor.stride = actor.stride.saturating_sub(1);
        actor.jump_wait = actor.jump_wait.saturating_sub(1);
        actor.knocked = actor.knocked.saturating_sub(1);
        let ground = [4, 5].map(|i| {
            solid(at(cells, w, h, actor.joints[i].x, actor.joints[i].y + 1.))
                || solid(at(cells, w, h, actor.joints[i].x, actor.joints[i].y))
        });
        if !actor.dead && actor.kind == Material::Fighter {
            if ground[0] || ground[1] {
                actor.age = 0;
            } else if actor.age > 50 {
                actor.knocked = 12;
            }
        }
        let mut direction = 0.;
        let mut jump = false;
        if !actor.dead && actor.knocked == 0 {
            if actor.kind == Material::Player {
                direction = (input.right as i8 - input.left as i8) as f32;
                jump = input.jump;
            } else {
                // Fighters hop with whichever foot is planted, rather than pursuing like a platformer enemy.
                let seed = tick
                    .wrapping_mul(1664525)
                    .wrapping_add(index as u32)
                    .wrapping_mul(1013904223);
                if (ground[0] || ground[1]) && seed % 100 < 10 {
                    direction = if seed & 256 == 0 { -1. } else { 1. };
                    let foot = if (actor.joints[4].x < actor.joints[5].x) == (direction > 0.) {
                        4
                    } else {
                        5
                    };
                    actor.joints[foot].x += direction * 4.;
                    actor.joints[foot].y -= 4.;
                }
            }
        }
        if !actor.dead && actor.kind == Material::Player {
            if actor.stride == 0 && (ground[0] || ground[1]) && direction != 0. {
                let knee = if (actor.joints[4].x < actor.joints[5].x) == (direction > 0.) {
                    2
                } else {
                    3
                };
                actor.joints[knee].x += direction * 3.2;
                actor.joints[knee].y -= 2.4;
                actor.stride = 15;
            } else if direction != 0. {
                actor.joints[1].x += direction * 0.1;
            }
            if jump && actor.jump_wait == 0 && (ground[0] || ground[1]) {
                for foot in [4, 5] {
                    actor.joints[foot].old_y += 6.;
                }
                actor.jump_wait = 50;
            }
        }
        let count = if actor.dead { 11 } else { 6 };
        for (i, joint) in actor.joints[..count].iter_mut().enumerate() {
            let (wx, wy, _) = air.sample(joint.x, joint.y);
            let gravity = if actor.dead || actor.knocked > 0 {
                0.1
            } else {
                [-0.2, -0.1, 0., 0., 0.3, 0.3][i]
            };
            let dx = ((joint.x - joint.old_x) * 0.995 + wx * 0.1).clamp(-3.8, 3.8);
            let dy = ((joint.y - joint.old_y) * 0.995 + gravity + wy * 0.1).clamp(-3.8, 3.8);
            joint.old_x = joint.x;
            joint.old_y = joint.y;
            joint.x += dx;
            joint.y += dy;
        }
        let edges = if actor.dead {
            [(1, 2, 4.), (3, 5, 4.), (4, 7, 4.), (6, 9, 5.), (8, 10, 5.)]
        } else {
            [(0, 1, 4.), (1, 2, 4.), (1, 3, 4.), (2, 4, 5.), (3, 5, 5.)]
        };
        let shrink = if actor.dead {
            (150_u16.saturating_sub(actor.age)) as f32 / 150.
        } else {
            1.
        };
        for _ in 0..3 {
            for (a, b, length) in edges {
                constrain(&mut actor.joints, a, b, length * shrink, 0.65);
            }
            if !actor.dead {
                constrain(&mut actor.joints, 2, 3, 5., 0.15);
            }
        }
        for i in 0..count {
            let joint = &mut actor.joints[i];
            // Sweep every endpoint so an energetic kick cannot pass a one-pixel platform.
            if !actor.dead && i < 4 {
                joint.x = joint.x.clamp(1., w as f32 - 2.);
                joint.y = joint.y.clamp(1., h as f32 - 2.);
                if matches!(
                    at(cells, w, h, joint.x, joint.y),
                    Material::Fire
                        | Material::Magma
                        | Material::Acid
                        | Material::Spark
                        | Material::Thunder
                        | Material::Laser
                ) {
                    actor.health = 0;
                }
                continue;
            }
            let ox = joint.old_x;
            let oy = joint.old_y;
            let dx = joint.x - ox;
            let dy = joint.y - oy;
            let steps = dx.abs().max(dy.abs()).ceil().max(1.) as usize;
            let (mut x, mut y) = (ox, oy);
            for _ in 0..steps {
                let nx = (x + dx / steps as f32).clamp(1., w as f32 - 2.);
                let ny = (y + dy / steps as f32).clamp(1., h as f32 - 2.);
                if !solid(at(cells, w, h, nx, y)) {
                    x = nx;
                }
                if !solid(at(cells, w, h, x, ny)) {
                    y = ny;
                }
            }
            joint.x = x;
            joint.y = y;
            if (x - ox - dx).abs() > 0.05 {
                joint.old_x = x;
            }
            if (y - oy - dy).abs() > 0.05 {
                joint.old_y = y;
                joint.old_x = x - (x - ox) * 0.5;
            }
            if !actor.dead
                && matches!(
                    at(cells, w, h, x, y),
                    Material::Fire
                        | Material::Magma
                        | Material::Acid
                        | Material::Spark
                        | Material::Thunder
                        | Material::Laser
                )
            {
                actor.health = 0;
            }
        }
        let hip = if actor.dead { 0 } else { 1 };
        actor.x = actor.joints[hip].x;
        actor.y = actor.joints[hip].y;
        actor.vx = actor.joints[hip].x - actor.joints[hip].old_x;
        actor.vy = actor.joints[hip].y - actor.joints[hip].old_y;
        if !actor.dead && actor.health == 0 {
            actor.fragment();
        }
    }
    // Foot-to-head contact is what transfers a fighter's kick into a loose ragdoll.
    for i in 0..actors.len() {
        if actors[i].dead || actors[i].kind != Material::Fighter {
            continue;
        }
        for j in 0..actors.len() {
            if i == j || actors[j].dead {
                continue;
            }
            for foot in [4, 5] {
                let f = actors[i].joints[foot];
                let head = actors[j].joints[0];
                if (f.x - head.x).abs() <= 2. && (0. ..=6.).contains(&(f.y - head.y)) {
                    actors[j].impulse(f.x - f.old_x, (f.y - f.old_y) * 2.);
                }
            }
        }
    }
    actors.retain(|a| !a.dead || a.age <= 150);
}
pub fn render(actors: &[Actor], pixels: &mut [u32], w: usize, h: usize) {
    fn line(p: &mut [u32], w: usize, h: usize, a: Joint, b: Joint, c: u32) {
        let (ax, ay, bx, by) = (
            a.x.round() as i32,
            a.y.round() as i32,
            b.x.round() as i32,
            b.y.round() as i32,
        );
        let steps = (bx - ax).abs().max((by - ay).abs()).max(1);
        for n in 0..=steps {
            let x = ax + (bx - ax) * n / steps;
            let y = ay + (by - ay) * n / steps;
            if x >= 0 && y >= 0 && x < w as i32 && y < h as i32 {
                p[y as usize * w + x as usize] = c;
            }
        }
    }
    for a in actors {
        let edges = if a.dead {
            [(1, 2), (3, 5), (4, 7), (6, 9), (8, 10)]
        } else {
            [(0, 1), (1, 2), (1, 3), (2, 4), (3, 5)]
        };
        for (i, (x, y)) in edges.into_iter().enumerate() {
            if !a.dead && i == 0 && a.kind == Material::Player {
                continue;
            }
            if a.dead && a.age > 145 - i as u16 * 5 {
                continue;
            }
            line(
                pixels,
                w,
                h,
                a.joints[x],
                a.joints[y],
                if !a.dead && i == 0 {
                    0xffe0ae
                } else {
                    0xffffff
                },
            );
        }
        if a.dead && a.age > 125 {
            continue;
        }
        let head = a.joints[0];
        let (min, max, top, bottom) = if a.dead {
            (-1, 1, -1, 1)
        } else if a.kind == Material::Player {
            (-2, 2, -1, 3)
        } else {
            (-1, 1, -1, 1)
        };
        for dy in top..=bottom {
            for dx in min..=max {
                if a.kind == Material::Player && dx > min && dx < max && dy > top && dy < bottom {
                    continue;
                }
                let x = head.x.round() as i32 + dx;
                let y = head.y.round() as i32 + dy;
                if x >= 0 && y >= 0 && x < w as i32 && y < h as i32 {
                    pixels[y as usize * w + x as usize] = 0xffe0ae;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scene() -> (Vec<Actor>, Vec<Cell>, Air) {
        let mut cells = vec![Cell::default(); 64 * 64];
        for x in 0..64 {
            cells[50 * 64 + x] = Cell::new(Material::Wall);
        }
        (
            vec![Actor::new(32., 41., Material::Player)],
            cells,
            Air::new(64, 64),
        )
    }
    #[test]
    fn elimination_detaches_limbs_and_retains_them_for_150_updates() {
        let (mut actors, cells, air) = scene();
        actors[0].health = 0;
        step(&mut actors, &cells, 64, 64, &air, Input::default(), 0);
        assert!(actors[0].dead);
        let mut pixels = vec![0; 4096];
        render(&actors, &mut pixels, 64, 64);
        assert!(pixels.contains(&0xffffff));
        // Head and hip copies cease to share a joint; there is no fireball substitution.
        for tick in 1..20 {
            step(&mut actors, &cells, 64, 64, &air, Input::default(), tick);
        }
        assert_ne!(actors[0].joints[2].x, actors[0].joints[3].x);
        for tick in 20..150 {
            step(&mut actors, &cells, 64, 64, &air, Input::default(), tick);
        }
        assert_eq!(actors.len(), 1);
        for tick in 150..152 {
            step(&mut actors, &cells, 64, 64, &air, Input::default(), tick);
        }
        assert!(actors.is_empty());
    }
    #[test]
    fn player_has_hollow_head_and_white_jointed_legs_without_arms() {
        let a = Actor::new(32., 24., Material::Player);
        let mut pixels = vec![0; 4096];
        render(&[a], &mut pixels, 64, 64);
        assert_eq!(pixels[20 * 64 + 32], 0); // hollow head centre
        assert_eq!(pixels[19 * 64 + 30], 0xffe0ae);
        assert_eq!(pixels[26 * 64 + 28], 0); // old invented arm location
        assert!(pixels[24 * 64..34 * 64].contains(&0xffffff));
    }
    #[test]
    fn explosive_impulse_reaches_all_joints_and_eliminates_player() {
        let mut a = Actor::new(32., 24., Material::Player);
        a.impulse(3., -2.);
        for joint in &a.joints[..6] {
            assert_eq!(joint.x - joint.old_x, 3.);
            assert_eq!(joint.y - joint.old_y, -2.);
        }
        assert_eq!(a.health, 0);
    }
    #[test]
    fn long_running_fighter_motion_remains_finite() {
        let (mut actors, cells, air) = scene();
        actors[0].kind = Material::Fighter;
        actors.push(Actor::new(40., 41., Material::Fighter));
        for tick in 0..1500 {
            step(&mut actors, &cells, 64, 64, &air, Input::default(), tick);
        }
        for actor in actors {
            for joint in &actor.joints[..6] {
                assert!(joint.x.is_finite() && joint.y.is_finite());
            }
        }
    }
}
