mod font;
mod physics;
mod sim;

use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
    mouse::MouseButton,
    pixels::{Color, PixelFormatEnum},
    rect::{Point, Rect},
    render::Canvas,
    video::Window,
};
use sim::{Input, Material, World};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

const WIDTH: usize = 400;
const HEIGHT: usize = 300;
const VIEW_W: u32 = 1200;
const VIEW_H: u32 = 800;
fn field() -> Rect {
    Rect::new(220, 64, 960, 720)
}
static QUIT: AtomicBool = AtomicBool::new(false);
static CAPTURE: AtomicBool = AtomicBool::new(false);
extern "C" fn request_capture(_: i32) {
    CAPTURE.store(true, Ordering::Relaxed);
}
extern "C" fn terminate(_: i32) {
    QUIT.store(true, Ordering::Relaxed);
}

type Render = Canvas<Window>;
fn rgb(value: u32) -> Color {
    Color::RGB((value >> 16) as u8, (value >> 8) as u8, value as u8)
}
fn text(c: &mut Render, x: i32, y: i32, s: &str, color: u32, scale: u32) {
    c.set_draw_color(rgb(color));
    for (i, ch) in s.chars().enumerate() {
        for (row, bits) in font::glyph(ch).iter().enumerate() {
            for col in 0..5 {
                if bits & (1 << (4 - col)) != 0 {
                    let _ = c.fill_rect(Rect::new(
                        x + i as i32 * 6 * scale as i32 + col * scale as i32,
                        y + row as i32 * scale as i32,
                        scale,
                        scale,
                    ));
                }
            }
        }
    }
}
fn button(c: &mut Render, rect: Rect, label: &str, color: u32, active: bool) {
    c.set_draw_color(rgb(if active { 0x263342 } else { 0x111722 }));
    let _ = c.fill_rect(rect);
    c.set_draw_color(rgb(if active { color } else { 0x293445 }));
    let _ = c.draw_rect(rect);
    let scale = if label.len() > 7 { 1 } else { 2 };
    let tx = rect.x() + (rect.width() as i32 - label.len() as i32 * 6 * scale as i32) / 2;
    text(
        c,
        tx,
        rect.y() + (rect.height() as i32 - 7 * scale as i32) / 2,
        label,
        color,
        scale,
    );
}
#[derive(Clone, Copy, PartialEq)]
enum Action {
    Select(Material),
    Pause,
    Glow,
    Demo,
    Save,
    Clear,
    Smaller,
    Bigger,
    Left,
    Right,
    Jump,
    PreviousPage,
    NextPage,
    Flow,
    SecondaryMode,
    PressureSign,
}
const PAGE_SIZE: usize = 20;
fn page_count() -> usize {
    Material::ALL.len().div_ceil(PAGE_SIZE)
}
fn controls(page: usize) -> Vec<(Rect, Action)> {
    let mut v: Vec<_> = Material::ALL
        .iter()
        .skip(page * PAGE_SIZE)
        .take(PAGE_SIZE)
        .enumerate()
        .map(|(i, m)| {
            (
                Rect::new(16 + (i % 2) as i32 * 96, 126 + (i / 2) as i32 * 46, 90, 40),
                Action::Select(*m),
            )
        })
        .collect();
    for (rect, action) in [
        (Rect::new(320, 14, 120, 32), Action::SecondaryMode),
        (Rect::new(16, 94, 40, 24), Action::PreviousPage),
        (Rect::new(162, 94, 40, 24), Action::NextPage),
        (Rect::new(1080, 76, 84, 32), Action::Flow),
        (Rect::new(960, 76, 110, 32), Action::PressureSign),
        (Rect::new(16, 594, 90, 40), Action::Pause),
        (Rect::new(112, 594, 90, 40), Action::Glow),
        (Rect::new(16, 640, 90, 40), Action::Demo),
        (Rect::new(112, 640, 90, 40), Action::Save),
        (Rect::new(16, 686, 186, 40), Action::Clear),
        (Rect::new(16, 744, 44, 40), Action::Smaller),
        (Rect::new(158, 744, 44, 40), Action::Bigger),
        (Rect::new(236, 720, 66, 48), Action::Left),
        (Rect::new(310, 720, 80, 48), Action::Jump),
        (Rect::new(398, 720, 66, 48), Action::Right),
    ] {
        v.push((rect, action));
    }
    v
}
fn state_path() -> PathBuf {
    let dir = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| ".".into()))
                .join(".local/state")
        });
    dir.join("surface-powder/world.bin")
}
fn save_world(w: &World, path: &PathBuf) -> String {
    match path
        .parent()
        .map(std::fs::create_dir_all)
        .transpose()
        .and_then(|_| w.save(path))
    {
        Ok(_) => "SAVED".into(),
        Err(e) => {
            eprintln!("save failed: {e}");
            "SAVE FAILED".into()
        }
    }
}
#[derive(Clone, Copy)]
struct Stroke {
    material: Material,
    last: (i32, i32),
    min: (i32, i32),
    max: (i32, i32),
    direction: (i32, i32),
    pressure: f32,
}
impl Stroke {
    fn new(material: Material, p: (i32, i32), negative: bool) -> Self {
        Self {
            material,
            last: p,
            min: p,
            max: p,
            direction: (1, 0),
            pressure: if negative { -0.8 } else { 0.8 },
        }
    }
    fn circle(&self, brush: i32) -> (i32, i32, i32) {
        (
            (self.min.0 + self.max.0) / 2,
            (self.min.1 + self.max.1) / 2,
            ((self.max.0 - self.min.0).max(self.max.1 - self.min.1) / 2)
                .max(brush + 4)
                .clamp(3, 48),
        )
    }
}
struct App {
    world: World,
    material: Material,
    secondary: Material,
    select_secondary: bool,
    negative_pressure: bool,
    page: usize,
    radius: i32,
    paused: bool,
    glow: bool,
    flow: bool,
    pointers: HashMap<i64, Stroke>,
    mouse: Option<(i32, i32)>,
    erasing: bool,
    held: HashMap<i64, Action>,
    notice: String,
    notice_until: Instant,
    clear_until: Option<Instant>,
}
fn one_shot(m: Material) -> bool {
    m.is_object() || matches!(m, Material::Player | Material::Fighter)
}
impl App {
    fn new(world: World) -> Self {
        Self {
            world,
            material: Material::Sand,
            secondary: Material::Empty,
            select_secondary: false,
            negative_pressure: false,
            page: 0,
            radius: 3,
            paused: false,
            glow: true,
            flow: false,
            pointers: HashMap::new(),
            mouse: None,
            erasing: false,
            held: HashMap::new(),
            notice: "DRAW SOMETHING".into(),
            notice_until: Instant::now() + Duration::from_secs(4),
            clear_until: None,
        }
    }
    fn notify(&mut self, s: String) {
        self.notice = s;
        self.notice_until = Instant::now() + Duration::from_secs(3);
    }
    fn change_page(&mut self, delta: isize) {
        self.page = (self.page as isize + delta).rem_euclid(page_count() as isize) as usize;
    }
    fn pour(&mut self, stroke: Stroke) {
        let p = stroke.last;
        match stroke.material {
            Material::Air => self
                .world
                .apply_pressure(p.0, p.1, self.radius * 3, stroke.pressure),
            Material::Wind => self.world.paint_line(
                p.0,
                p.1,
                p.0 + stroke.direction.0,
                p.1 + stroke.direction.1,
                self.radius,
                Material::Wind,
            ),
            Material::Drag => {}
            m if one_shot(m) => {}
            m => self.world.paint_line(p.0, p.1, p.0, p.1, self.radius, m),
        }
    }
    fn press(&mut self, id: i64, x: i32, y: i32, buttons: &[(Rect, Action)], path: &PathBuf) {
        self.clear_until = self.clear_until.filter(|t| *t > Instant::now());
        for (r, a) in buttons {
            if *a == Action::PressureSign && self.material != Material::Air {
                continue;
            }
            if matches!(a, Action::Left | Action::Right | Action::Jump)
                && self.world.actors.is_empty()
            {
                continue;
            }
            if r.contains_point(Point::new(x, y)) {
                match *a {
                    Action::Select(m) => {
                        if self.select_secondary && !m.is_cell() {
                            self.notify("CHOOSE A MATERIAL".into());
                        } else if (self.erasing || self.select_secondary) && m.is_cell() {
                            self.secondary = m;
                            if m != Material::Empty {
                                self.world.brush_payload = m;
                            }
                            self.select_secondary = false;
                            self.notify(format!("SECONDARY: {}", m.name()));
                        } else {
                            self.material = m;
                        }
                    }
                    Action::Pause => self.paused = !self.paused,
                    Action::Glow => self.glow = !self.glow,
                    Action::Flow => self.flow = !self.flow,
                    Action::PressureSign => self.negative_pressure = !self.negative_pressure,
                    Action::SecondaryMode => {
                        self.select_secondary = !self.select_secondary;
                        if self.select_secondary {
                            self.notify("CHOOSE R MATERIAL".into());
                        }
                    }
                    Action::PreviousPage => self.change_page(-1),
                    Action::NextPage => self.change_page(1),
                    Action::Demo => {
                        self.world.seed_demo();
                        self.notify("DEMO LOADED".into());
                    }
                    Action::Save => {
                        let s = save_world(&self.world, path);
                        self.notify(s);
                    }
                    Action::Clear => {
                        if self.clear_until.is_some() {
                            self.world.clear();
                            self.clear_until = None;
                        } else {
                            self.clear_until = Some(Instant::now() + Duration::from_secs(3));
                            self.notify("CLEAR AGAIN?".into());
                        }
                    }
                    Action::Smaller => self.radius = (self.radius - 1).max(1),
                    Action::Bigger => self.radius = (self.radius + 1).min(16),
                    Action::Left | Action::Right | Action::Jump => {
                        self.held.insert(id, *a);
                    }
                }
                return;
            }
        }
        if let Some(p) = grid(x, y) {
            let m = if self.erasing && self.material != Material::Air {
                self.secondary
            } else {
                self.material
            };
            let stroke = Stroke::new(m, p, self.erasing || self.negative_pressure);
            if one_shot(m) && m != Material::Bubble {
                self.world.paint_line(
                    p.0,
                    p.1,
                    p.0,
                    p.1,
                    if m.is_object() {
                        self.radius + 5
                    } else {
                        self.radius
                    },
                    m,
                );
            } else if m != Material::Bubble {
                self.pour(stroke);
            }
            self.pointers.insert(id, stroke);
        }
    }
    fn drag(&mut self, id: i64, x: i32, y: i32) {
        if let Some(p) = grid(x, y) {
            if let Some(stroke) = self.pointers.get_mut(&id) {
                let prev = stroke.last;
                let dx = p.0 - prev.0;
                let dy = p.1 - prev.1;
                stroke.min = (stroke.min.0.min(p.0), stroke.min.1.min(p.1));
                stroke.max = (stroke.max.0.max(p.0), stroke.max.1.max(p.1));
                if dx != 0 || dy != 0 {
                    stroke.direction = (dx.clamp(-12, 12), dy.clamp(-12, 12));
                }
                match stroke.material {
                    Material::Drag => {
                        self.world
                            .drag_at(prev.0, prev.1, dx, dy, self.radius * 2 + 5)
                    }
                    Material::Air => {
                        self.world
                            .apply_pressure(p.0, p.1, self.radius * 3, stroke.pressure)
                    }
                    m if !one_shot(m) => {
                        self.world
                            .paint_line(prev.0, prev.1, p.0, p.1, self.radius, m)
                    }
                    _ => {}
                }
                stroke.last = p;
            }
        }
    }
    fn release(&mut self, id: i64) {
        if let Some(stroke) = self.pointers.remove(&id) {
            if stroke.material == Material::Bubble {
                let (x, y, r) = stroke.circle(self.radius);
                self.world.paint_line(x, y, x, y, r, Material::Bubble);
            }
        }
        self.held.remove(&id);
    }
}
fn grid(x: i32, y: i32) -> Option<(i32, i32)> {
    field().contains_point(Point::new(x, y)).then(|| {
        (
            (x - field().x()) * WIDTH as i32 / field().width() as i32,
            (y - field().y()) * HEIGHT as i32 / field().height() as i32,
        )
    })
}
fn touch_point(x: f32, y: f32) -> (i32, i32) {
    // SDL_RenderSetLogicalSize has already normalized touch to the logical viewport.
    ((x * VIEW_W as f32) as i32, (y * VIEW_H as f32) as i32)
}
fn draw(
    app: &App,
    c: &mut Render,
    texture: &sdl2::render::Texture,
    buttons: &[(Rect, Action)],
    count: usize,
) -> Result<(), String> {
    c.set_draw_color(rgb(0x090d14));
    c.clear();
    text(c, 16, 18, "SURFACE POWDER", 0x72ffc0, 3);
    text(c, 680, 23, "TAB: TOOLS  G: GLOW  F: FLOW", 0x8294ad, 2);
    text(c, 16, 68, app.material.name(), app.material.color(), 2);
    if matches!(app.material, Material::Firework | Material::Ball) {
        text(
            c,
            236,
            48,
            &format!("PAYLOAD: {}", app.world.brush_payload.name()),
            0xd697f5,
            1,
        );
    }
    text(
        c,
        16,
        48,
        &format!("R: {}", app.secondary.name()),
        0xd697f5,
        1,
    );
    text(
        c,
        72,
        101,
        &format!("{}/{}", app.page + 1, page_count()),
        0x8294ad,
        1,
    );
    c.set_draw_color(rgb(0x354050));
    c.draw_rect(Rect::new(218, 62, 964, 724))?;
    c.copy(texture, None, field())?;
    text(
        c,
        236,
        76,
        &format!("{} DOTS  {} OBJECTS", count, app.world.bodies.len()),
        0x8294ad,
        1,
    );
    if app.flow || matches!(app.material, Material::Wind | Material::Air | Material::Fan) {
        for y in (8..HEIGHT).step_by(16) {
            for x in (8..WIDTH).step_by(16) {
                let (vx, vy, p) = app.world.air.sample(x as f32, y as f32);
                if vx.abs() + vy.abs() + p.abs() < 0.03 {
                    continue;
                }
                c.set_draw_color(rgb(if p > 0.1 {
                    0x3ea784
                } else if p < -0.1 {
                    0x5e72c9
                } else {
                    0x315c6b
                }));
                let sx = field().x() + (x as f32 * 2.4) as i32;
                let sy = field().y() + (y as f32 * 2.4) as i32;
                c.draw_line(
                    Point::new(sx, sy),
                    Point::new(
                        sx + (vx * 8.).clamp(-18., 18.) as i32,
                        sy + (vy * 8.).clamp(-18., 18.) as i32,
                    ),
                )?;
                c.fill_rect(Rect::new(sx, sy, 2, 2))?;
            }
        }
    }
    for stroke in app
        .pointers
        .values()
        .filter(|s| s.material == Material::Bubble)
    {
        let (x, y, r) = stroke.circle(app.radius);
        c.set_draw_color(rgb(0x70e6ef));
        for n in 0..48 {
            let a = n as f32 * std::f32::consts::TAU / 48.;
            let b = (n + 1) as f32 * std::f32::consts::TAU / 48.;
            let point = |angle: f32| {
                Point::new(
                    field().x() + ((x as f32 + angle.cos() * r as f32) * 2.4) as i32,
                    field().y() + ((y as f32 + angle.sin() * r as f32) * 2.4) as i32,
                )
            };
            c.draw_line(point(a), point(b))?;
        }
    }
    for (rect, action) in buttons {
        if *action == Action::PressureSign && app.material != Material::Air {
            continue;
        }
        let (label, color, active) = match action {
            Action::Select(m) => (
                m.name(),
                if *m == Material::Empty {
                    0xb5c4d9
                } else {
                    m.color()
                },
                app.material == *m,
            ),
            Action::Pause => (
                if app.paused { "RESUME" } else { "PAUSE" },
                0xb5c4d9,
                app.paused,
            ),
            Action::Glow => ("GLOW", 0xc6a1ff, app.glow),
            Action::Demo => ("DEMO", 0xb5c4d9, false),
            Action::Save => ("SAVE", 0xb5c4d9, false),
            Action::Clear => (
                if app.clear_until.is_some() {
                    "CLEAR AGAIN?"
                } else {
                    "CLEAR"
                },
                0xff8e91,
                false,
            ),
            Action::Smaller => ("-", 0xb5c4d9, false),
            Action::Bigger => ("+", 0xb5c4d9, false),
            Action::Left => ("<", 0x72ffc0, app.held.values().any(|a| a == action)),
            Action::Right => (">", 0x72ffc0, app.held.values().any(|a| a == action)),
            Action::Jump => ("JUMP", 0x72ffc0, app.held.values().any(|a| a == action)),
            Action::PreviousPage => ("<", 0xb5c4d9, false),
            Action::NextPage => (">", 0xb5c4d9, false),
            Action::Flow => ("FLOW", 0x70e6ef, app.flow),
            Action::PressureSign => (
                if app.negative_pressure {
                    "AIR -"
                } else {
                    "AIR +"
                },
                0x70e6ef,
                app.negative_pressure,
            ),
            Action::SecondaryMode => (
                if app.select_secondary {
                    "PICK R"
                } else {
                    "SET R"
                },
                0xd697f5,
                app.select_secondary,
            ),
        };
        if matches!(action, Action::Left | Action::Right | Action::Jump)
            && app.world.actors.is_empty()
        {
            continue;
        }
        button(c, *rect, label, color, active);
        if matches!(action,Action::Select(m) if *m==app.secondary && *m!=Material::Empty) {
            c.set_draw_color(rgb(0xd697f5));
            c.fill_rect(Rect::new(rect.right() - 7, rect.y() + 3, 4, 4))?;
        }
    }
    text(c, 73, 758, &format!("PEN {}", app.radius), 0x8294ad, 1);
    if Instant::now() < app.notice_until {
        c.set_draw_color(rgb(0x111722));
        c.fill_rect(Rect::new(750, 116, 420, 30))?;
        text(c, 764, 124, &app.notice, 0x72ffc0, 2);
    }
    Ok(())
}
fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|s| s == "--benchmark") {
        return benchmark();
    }
    // Install termination handler so a hardware-button switch saves the scene.
    unsafe {
        sdl2::libc::signal(sdl2::libc::SIGTERM, terminate as *const () as usize);
        sdl2::libc::signal(sdl2::libc::SIGUSR1, request_capture as *const () as usize);
        sdl2::libc::signal(sdl2::libc::SIGINT, terminate as *const () as usize);
    }
    sdl2::hint::set("SDL_VIDEO_WAYLAND_ALLOW_LIBDECOR", "0");
    sdl2::hint::set("SDL_TOUCH_MOUSE_EVENTS", "0");
    sdl2::hint::set("SDL_MOUSE_TOUCH_EVENTS", "0");
    sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "0");
    let ctx = sdl2::init()?;
    let video = ctx.video()?;
    eprintln!("powder: video ready ({})", video.current_video_driver());
    let window = video
        .window("Surface Powder", VIEW_W, VIEW_H)
        .position_centered()
        .resizable()
        .fullscreen_desktop()
        .allow_highdpi()
        .build()
        .map_err(|e| e.to_string())?;
    eprintln!("powder: fullscreen window ready");
    let mut c = window.into_canvas().build().map_err(|e| e.to_string())?;
    eprintln!("powder: renderer ready ({})", c.info().name);
    c.set_logical_size(VIEW_W, VIEW_H)
        .map_err(|e| e.to_string())?;
    let creator = c.texture_creator();
    let mut texture = creator
        .create_texture_streaming(PixelFormatEnum::RGB888, WIDTH as u32, HEIGHT as u32)
        .map_err(|e| e.to_string())?;
    let path = state_path();
    let world = match World::load(&path) {
        Ok(w) if w.width == WIDTH && w.height == HEIGHT => w,
        Ok(_) => return Err("Saved world has unexpected dimensions".into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let mut w = World::new(WIDTH, HEIGHT);
            w.seed_demo();
            w
        }
        Err(e) => return Err(format!("Save could not be loaded (preserved): {e}")),
    };
    let mut app = App::new(world);
    let mut events = ctx.event_pump()?;
    let mut pixels = vec![0u32; WIDTH * HEIGHT];
    let mut trails = vec![0u32; WIDTH * HEIGHT];
    let mut last_save = Instant::now();
    let mut last_step = Instant::now();
    let mut frames = 0u64;
    let max_frames = args
        .iter()
        .position(|s| s == "--frames")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<u64>().ok());
    let screenshot = args
        .iter()
        .position(|s| s == "--screenshot")
        .and_then(|i| args.get(i + 1));
    let mut counts = 0;
    eprintln!("powder: scene loaded; entering main loop");
    'running: while !QUIT.load(Ordering::Relaxed) {
        let frame = Instant::now();
        for e in events.poll_iter() {
            match e {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    repeat: false,
                    ..
                } => app.paused = !app.paused,
                Event::KeyDown {
                    keycode: Some(Keycode::G),
                    repeat: false,
                    ..
                } => app.glow = !app.glow,
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    repeat: false,
                    ..
                } => app.flow = !app.flow,
                Event::KeyDown {
                    keycode: Some(Keycode::Tab | Keycode::PageDown),
                    repeat: false,
                    ..
                } => app.change_page(1),
                Event::KeyDown {
                    keycode: Some(Keycode::PageUp),
                    repeat: false,
                    ..
                } => app.change_page(-1),
                Event::MouseButtonDown {
                    x, y, mouse_btn, ..
                } => {
                    app.erasing = mouse_btn == MouseButton::Right;
                    app.mouse = Some((x, y));
                    app.press(-1, x, y, &controls(app.page), &path);
                }
                Event::MouseButtonUp { .. } => {
                    app.release(-1);
                    app.mouse = None;
                    app.erasing = false;
                }
                Event::MouseMotion { x, y, .. } if app.mouse.is_some() => {
                    app.mouse = Some((x, y));
                    app.drag(-1, x, y);
                }
                Event::MouseWheel { y, .. } => app.radius = (app.radius + y).clamp(1, 16),
                Event::FingerDown {
                    finger_id, x, y, ..
                } => {
                    let p = touch_point(x, y);
                    app.press(finger_id, p.0, p.1, &controls(app.page), &path);
                }
                Event::FingerMotion {
                    finger_id, x, y, ..
                } => {
                    let p = touch_point(x, y);
                    app.drag(finger_id, p.0, p.1);
                }
                Event::FingerUp { finger_id, .. } => app.release(finger_id),
                Event::Window {
                    win_event: sdl2::event::WindowEvent::FocusLost,
                    ..
                } => {
                    app.pointers.clear();
                    app.held.clear();
                    app.mouse = None;
                    app.erasing = false;
                }
                _ => {}
            }
        }
        let keyboard = events.keyboard_state();
        let input = Input {
            left: keyboard.is_scancode_pressed(Scancode::Left)
                || keyboard.is_scancode_pressed(Scancode::A)
                || app.held.values().any(|a| *a == Action::Left),
            right: keyboard.is_scancode_pressed(Scancode::Right)
                || keyboard.is_scancode_pressed(Scancode::D)
                || app.held.values().any(|a| *a == Action::Right),
            jump: keyboard.is_scancode_pressed(Scancode::Up)
                || keyboard.is_scancode_pressed(Scancode::W)
                || app.held.values().any(|a| *a == Action::Jump),
        };
        let step_time = Duration::from_nanos(16_666_667);
        if app.paused {
            last_step = frame;
        } else {
            // Fixed simulation clock matches the original's 60 Hz independently of presentation.
            // Bound catch-up so a stalled renderer cannot queue unbounded physics work.
            let mut steps = 0;
            while frame.duration_since(last_step) >= step_time && steps < 4 {
                let strokes: Vec<_> = app.pointers.values().copied().collect();
                for stroke in strokes {
                    app.pour(stroke);
                }
                app.world.step(input);
                last_step += step_time;
                steps += 1;
            }
            if steps == 4 {
                last_step = frame;
            }
        }
        app.world.render(&mut pixels);
        for (p, t) in pixels.iter_mut().zip(&mut trails) {
            if app.glow {
                let faded = ((((*t >> 16) & 255) * 4 / 5) << 16)
                    | ((((*t >> 8) & 255) * 4 / 5) << 8)
                    | ((*t & 255) * 4 / 5);
                *p = ((*p & 0xff0000).max(faded & 0xff0000))
                    | ((*p & 0xff00).max(faded & 0xff00))
                    | ((*p & 255).max(faded & 255));
            }
            *t = *p;
        }
        texture.with_lock(None, |buf, pitch| {
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let offset = y * pitch + x * 4;
                    buf[offset..offset + 4].copy_from_slice(&pixels[y * WIDTH + x].to_ne_bytes());
                }
            }
        })?;
        if frames % 30 == 0 {
            counts = app
                .world
                .cells
                .iter()
                .filter(|c| c.material != Material::Empty)
                .count();
        }
        if app.clear_until.is_some_and(|t| t < Instant::now()) {
            app.clear_until = None;
        }
        draw(&app, &mut c, &texture, &controls(app.page), counts)?;
        frames += 1;
        let finishing = max_frames.is_some_and(|n| frames >= n);
        if CAPTURE.swap(false, Ordering::Relaxed) || (finishing && screenshot.is_some()) {
            let file = screenshot
                .map(PathBuf::from)
                .unwrap_or_else(|| path.with_file_name("preview.ppm"));
            if let Some(parent) = file.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let (w, h) = c.output_size()?;
            let bytes = c.read_pixels(None, PixelFormatEnum::RGB24)?;
            use std::io::Write;
            let mut f = std::fs::File::create(file).map_err(|e| e.to_string())?;
            write!(f, "P6\n{w} {h}\n255\n")
                .and_then(|_| f.write_all(&bytes))
                .map_err(|e| e.to_string())?;
        }
        c.present();
        if finishing {
            break;
        }
        if last_save.elapsed() > Duration::from_secs(30) {
            let result = save_world(&app.world, &path);
            if result == "SAVE FAILED" {
                app.notify(result);
            }
            last_save = Instant::now();
        }
        std::thread::sleep(Duration::from_millis(33).saturating_sub(frame.elapsed()));
    }
    let result = save_world(&app.world, &path);
    if result == "SAVE FAILED" {
        return Err(result);
    }
    Ok(())
}
fn benchmark() -> Result<(), String> {
    let mut world = World::new(WIDTH, HEIGHT);
    world.seed_demo();
    let mut pixels = vec![0; WIDTH * HEIGHT];
    let start = Instant::now();
    for _ in 0..900 {
        world.step(Input::default());
        world.render(&mut pixels);
    }
    println!(
        "900 simulation+render steps: {:.3}s ({:.2}ms/frame); cells={} bytes",
        start.elapsed().as_secs_f64(),
        start.elapsed().as_secs_f64() * 1000. / 900.,
        std::mem::size_of_val(world.cells.as_slice())
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn touch_mapping_handles_letterboxing() {
        assert_eq!(touch_point(0.5, 0.5), (600, 400));
        assert_eq!(touch_point(0., 0.), (0, 0));
        assert_eq!(grid(220, 64), Some((0, 0)));
        assert_eq!(grid(1179, 783), Some((399, 299)));
        assert_eq!(grid(1180, 784), None);
    }
    #[test]
    fn hidden_actor_buttons_do_not_intercept_painting() {
        let mut app = App::new(World::new(WIDTH, HEIGHT));
        app.press(1, 250, 740, &controls(0), &PathBuf::from("unused"));
        let (x, y) = grid(250, 740).unwrap();
        assert_eq!(app.world.at(x, y), Material::Sand);
        assert!(app.held.is_empty());
    }
    #[test]
    fn all_palette_pages_are_reachable_without_overlapping_controls() {
        let mut seen = std::collections::HashSet::new();
        for page in 0..page_count() {
            let buttons = controls(page);
            for (r, a) in &buttons {
                assert!(
                    r.x() >= 0
                        && r.y() >= 0
                        && r.right() <= VIEW_W as i32
                        && r.bottom() <= VIEW_H as i32
                );
                if let Action::Select(m) = a {
                    assert!(!r.has_intersection(field()));
                    assert!(seen.insert(*m as u8));
                    for (other, other_action) in &buttons {
                        if !matches!(other_action, Action::Select(_)) {
                            assert!(!r.has_intersection(*other));
                        }
                    }
                }
            }
        }
        assert_eq!(seen.len(), Material::ALL.len());
    }
    #[test]
    fn bubble_gesture_creates_one_sized_bubble_on_release() {
        let mut app = App::new(World::new(WIDTH, HEIGHT));
        app.material = Material::Bubble;
        app.press(1, 700, 400, &controls(0), &PathBuf::from("unused"));
        app.drag(1, 772, 472);
        app.drag(1, 700, 400);
        assert!(app.world.bodies.is_empty());
        app.release(1);
        assert_eq!(app.world.bodies.len(), 1);
        assert_eq!(app.world.bodies[0].kind, physics::BodyKind::Bubble);
        assert!((app.world.bodies[0].radius - 15.).abs() < 1.);
    }
    #[test]
    fn holding_object_brush_does_not_create_a_body_every_frame() {
        let mut app = App::new(World::new(WIDTH, HEIGHT));
        app.material = Material::Box;
        app.press(1, 700, 400, &controls(0), &PathBuf::from("unused"));
        for _ in 0..30 {
            let stroke = app.pointers[&1];
            app.pour(stroke);
        }
        assert_eq!(app.world.bodies.len(), 1);
    }
    #[test]
    fn air_secondary_stroke_lowers_pressure() {
        let mut app = App::new(World::new(WIDTH, HEIGHT));
        app.material = Material::Air;
        app.erasing = true;
        app.press(1, 700, 400, &controls(0), &PathBuf::from("unused"));
        let (x, y) = grid(700, 400).unwrap();
        assert!(app.world.air.sample(x as f32, y as f32).2 < 0.);
        assert!(app
            .world
            .cells
            .iter()
            .all(|c| c.material == Material::Empty));
    }
    #[test]
    fn selecting_secondary_material_sets_fireworks_payload_without_switching_tool() {
        let mut app = App::new(World::new(WIDTH, HEIGHT));
        app.material = Material::Firework;
        app.erasing = true;
        let buttons = controls(0);
        let r = buttons
            .iter()
            .find(|(_, a)| *a == Action::Select(Material::Water))
            .unwrap()
            .0;
        app.press(-1, r.x() + 5, r.y() + 5, &buttons, &PathBuf::from("unused"));
        assert_eq!(app.material, Material::Firework);
        assert_eq!(app.secondary, Material::Water);
        app.erasing = false;
        app.press(1, 700, 400, &buttons, &PathBuf::from("unused"));
        let (x, y) = grid(700, 400).unwrap();
        let cell = app.world.cells[y as usize * WIDTH + x as usize];
        assert_eq!(cell.material, Material::Firework);
        assert_eq!(cell.life & 63, Material::Water as u8);
    }
    #[test]
    fn touch_can_choose_a_secondary_material_without_a_right_mouse_button() {
        let mut app = App::new(World::new(WIDTH, HEIGHT));
        let buttons = controls(0);
        let set_r = buttons
            .iter()
            .find(|(_, a)| *a == Action::SecondaryMode)
            .unwrap()
            .0;
        app.press(
            1,
            set_r.x() + 5,
            set_r.y() + 5,
            &buttons,
            &PathBuf::from("unused"),
        );
        assert!(app.select_secondary);
        let water = buttons
            .iter()
            .find(|(_, a)| *a == Action::Select(Material::Water))
            .unwrap()
            .0;
        app.press(
            2,
            water.x() + 5,
            water.y() + 5,
            &buttons,
            &PathBuf::from("unused"),
        );
        assert_eq!(app.secondary, Material::Water);
        assert_eq!(app.material, Material::Sand);
        assert_eq!(app.world.brush_payload, Material::Water);
        assert!(!app.select_secondary);
    }
    #[test]
    fn touch_can_switch_air_to_suction_without_a_mouse() {
        let mut app = App::new(World::new(WIDTH, HEIGHT));
        app.material = Material::Air;
        let buttons = controls(0);
        let toggle = buttons
            .iter()
            .find(|(_, a)| *a == Action::PressureSign)
            .unwrap()
            .0;
        app.press(
            1,
            toggle.x() + 5,
            toggle.y() + 5,
            &buttons,
            &PathBuf::from("unused"),
        );
        app.press(2, 700, 400, &buttons, &PathBuf::from("unused"));
        let (x, y) = grid(700, 400).unwrap();
        assert!(app.world.air.sample(x as f32, y as f32).2 < 0.);
    }
    #[test]
    fn upward_wind_gesture_sets_upward_air_velocity() {
        let mut app = App::new(World::new(WIDTH, HEIGHT));
        app.material = Material::Wind;
        app.press(1, 700, 400, &controls(0), &PathBuf::from("unused"));
        app.drag(1, 700, 300);
        let (x, y) = grid(700, 300).unwrap();
        assert!(app.world.air.sample(x as f32, y as f32).1 < 0.);
        assert!(app
            .world
            .cells
            .iter()
            .all(|c| c.material == Material::Empty));
    }
}
