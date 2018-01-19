extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};

#[derive(Copy, Clone)]
struct Platform {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct Corner {
    x: i8,
    y: i8,
}

impl Platform {
    fn corner_x(&self, corner: &Corner) -> i32 {
        self.x + (1 + corner.x as i32) * self.w / 2
    }
    fn corner_y(&self, corner: &Corner) -> i32 {
        self.y + (1 + corner.y as i32) * self.h / 2
    }
}

const CORNERS: [Corner; 8] = [
    Corner { x: -1, y: -1 },
    Corner { x: -1, y: 0 },
    Corner { x: -1, y: 1 },
    Corner { x: 0, y: -1 },
    Corner { x: 0, y: 1 },
    Corner { x: 1, y: -1 },
    Corner { x: 1, y: 0 },
    Corner { x: 1, y: 1 },
];

const HANDLE_SIZE: f64 = 8.0;
const MIN_PLATFORM_SIZE: i32 = 20;

#[derive(Copy, Clone)]
struct DragState {
    platform: usize,
    original_platform: Platform,
    ox: f64,
    oy: f64,
    corner: Option<Corner>,
}

fn apply_drag(drag: &DragState, mouse_x: f64, mouse_y: f64) -> Platform {
    let mut p = drag.original_platform;
    let mut dx = (mouse_x - drag.ox) as i32;
    let mut dy = (mouse_y - drag.oy) as i32;
    match drag.corner {
        Some(corner) => {
            if corner.x == -1 {
                if dx > p.w - MIN_PLATFORM_SIZE {
                    dx = p.w - MIN_PLATFORM_SIZE
                }
                p.x += dx;
                p.w -= dx;
            } else if corner.x == 1 {
                if dx < -(p.w - MIN_PLATFORM_SIZE) {
                    dx = -(p.w - MIN_PLATFORM_SIZE)
                }
                p.w += dx;
            }
            if corner.y == -1 {
                if dy > p.h - MIN_PLATFORM_SIZE {
                    dy = p.h - MIN_PLATFORM_SIZE
                }
                p.y += dy;
                p.h -= dy;
            } else if corner.y == 1 {
                if dy < -(p.h - MIN_PLATFORM_SIZE) {
                    dy = -(p.h - MIN_PLATFORM_SIZE)
                }
                p.h += dy;
            }
        }
        None => {
            p.x += dx;
            p.y += dy;
        }
    }
    p
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    proposed_drag: Option<DragState>,
    active_drag: Option<DragState>,
    platforms: Vec<Platform>,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];

        let plats = &self.platforms;
        let drag = self.active_drag.or(self.proposed_drag);
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);
            for (i, p) in plats.iter().enumerate() {
                rectangle(
                    RED,
                    rectangle::rectangle_by_corners(
                        f64::from(p.x),
                        f64::from(p.y),
                        f64::from(p.x + p.w),
                        f64::from(p.y + p.h),
                    ),
                    c.transform,
                    gl,
                );
                if let Some(drag) = drag {
                    if drag.platform == i {
                        for corner in CORNERS.iter() {
                            ellipse(
                                if Some(corner) == drag.corner.as_ref() { BLUE } else { YELLOW },
                                rectangle::centered_square(
                                    p.corner_x(corner) as f64,
                                    p.corner_y(corner) as f64,
                                    HANDLE_SIZE / 2.0,
                                ),
                                c.transform,
                                gl,
                            );
                        }
                    }
                }
            }
        });
    }

    fn update(&mut self, _args: &UpdateArgs) {}
}

fn point_in_plat(x: f64, y: f64, p: &Platform) -> bool {
    let rx = f64::from(p.x);
    let ry = f64::from(p.y);
    let rw = f64::from(p.w);
    let rh = f64::from(p.h);
    rx <= x && x <= rx + rw && ry <= y && y <= ry + rh
}

fn point_in_handle(x: f64, y: f64, p:&Platform, corner:&Corner) -> bool {
    let rx = p.corner_x(corner) as f64 - HANDLE_SIZE / 2.0;
    let ry = p.corner_y(corner) as f64 - HANDLE_SIZE / 2.0;
    rx <= x && x <= rx + HANDLE_SIZE && ry <= y && y <= ry + HANDLE_SIZE
}

fn prepare_drag(app: &App, mouse_x: f64, mouse_y: f64) -> Option<DragState> {
    match app.platforms.iter().rev().position(|p| {
        point_in_plat(mouse_x, mouse_y, p) ||
        CORNERS.iter().any(|corner| point_in_handle(mouse_x, mouse_y, p, corner))
    }) {
        Some(p) => {
            let corner = CORNERS.iter().find(|corner| point_in_handle(mouse_x, mouse_y, &app.platforms[p], corner));
            Some(DragState {
                platform: p,
                original_platform: app.platforms[p],
                ox: mouse_x,
                oy: mouse_y,
                corner: corner.cloned(),
            })
        }
        None => None,
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let w:i32 = 400;
    let h:i32 = 400;
    let mut window: Window = WindowSettings::new("dragging-square", [w as u32, h as u32])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        proposed_drag: None,
        active_drag: None,
        platforms: vec![
            Platform {
                x: (w / 2 - 25),
                y: (h / 2 - 25),
                w: 50,
                h: 50,
            },
        ],
    };

    let mut events = Events::new(EventSettings::new());
    let mut mouse_x = 0.0;
    let mut mouse_y = 0.0;
    let mut mouse_down = false;
    while let Some(e) = events.next(&mut window) {
        //Do stuff with the mouse events
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }

        e.mouse_cursor(|x, y| {
            mouse_x = x;
            mouse_y = y;
            if mouse_down {
                app.proposed_drag = None;
            } else {
                app.proposed_drag = prepare_drag(&app, mouse_x, mouse_y);
            }
        });
        e.press(|button| match button {
            Button::Mouse(MouseButton::Left) => {
                mouse_down = true;
                app.active_drag = prepare_drag(&app, mouse_x, mouse_y);
            }
            _ => {}
        });
        e.release(|button| match button {
            Button::Mouse(MouseButton::Left) => {
                mouse_down = false;
                app.active_drag = None;
                app.proposed_drag = prepare_drag(&app, mouse_x, mouse_y);
            }
            _ => {}
        });
        if let Some(drag) = app.active_drag.as_ref() {
            app.platforms[drag.platform] = apply_drag(drag, mouse_x, mouse_y);
        }
    }
}
