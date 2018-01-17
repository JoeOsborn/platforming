extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

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

struct DragState {
    platform: usize,
    original_platform: Platform,
    ox: f64,
    oy: f64,
}

fn apply_drag(drag: &DragState, mouse_x: f64, mouse_y: f64) -> Platform {
	Platform {
        x: drag.original_platform.x + (mouse_x - drag.ox) as i32,
        y: drag.original_platform.y + (mouse_y - drag.oy) as i32,
        ..drag.original_platform
    }
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    mouse_down: bool,
    mouse_x: f64,
    mouse_y: f64,
    drag: Option<DragState>,
    platforms: Vec<Platform>,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let plats = &self.platforms;
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);
            for p in plats {
                rectangle(
                    RED,
                    rectangle::rectangle_by_corners(
                        p.x as f64,
                        p.y as f64,
                        (p.x + p.w) as f64,
                        (p.y + p.h) as f64,
                    ),
                    c.transform,
                    gl,
                )
            }
        });
    }

    fn update(&mut self, _args: &UpdateArgs) {}
}

fn point_in_plat(x: f64, y: f64, p:&Platform) -> bool {
    let rx = p.x as f64;
    let ry = p.y as f64;
    let rw = p.w as f64;
    let rh = p.h as f64;
    rx <= x && x <= rx + rw && ry <= y && y <= ry + rh
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let w = 400;
    let h = 400;
    let mut window: Window = WindowSettings::new("dragging-square", [w, h])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        mouse_down: false,
        mouse_x: 0.0,
        mouse_y: 0.0,
        drag: None,
        platforms: vec![
            Platform {
                x: (w / 2 - 25) as i32,
                y: (h / 2 - 25) as i32,
                w: 50,
                h: 50,
            },
        ],
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        //Do stuff with the mouse events
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }

        e.mouse_cursor(|x, y| {
            app.mouse_x = x;
            app.mouse_y = y;
        });
        e.press(|button| match button {
            Button::Mouse(MouseButton::Left) => {
                app.mouse_down = true;
                app.drag = match app.platforms.iter().rev().position(|p| {
                    point_in_plat(app.mouse_x, app.mouse_y, p)
                }) {
                    Some(p) => {
                        Some(DragState {
                            platform: p,
                            original_platform: app.platforms[p],
                            ox: app.mouse_x,
                            oy: app.mouse_y,
                        })
                    }
                    None => None,
                }
            }
            _ => {}
        });
        e.release(|button| match button {
            Button::Mouse(MouseButton::Left) => {
                app.mouse_down = false;
                app.drag = None;
            }
            _ => {}
        });
        if let Some(drag) = app.drag.as_ref() {
            app.platforms[drag.platform] = apply_drag(drag, app.mouse_x, app.mouse_y);
        }
    }
}
