use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};

use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderEvent, UpdateEvent};
use piston::window::WindowSettings;

mod animation;
mod chess_controller;
mod chess_renderer;

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("Chess", [600, 600])
        .samples(1)
        .graphics_api(opengl)
        .exit_on_esc(false)
        .resizable(false)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut chess_renderer = chess_renderer::ChessRenderer::new(GlGraphics::new(opengl));
    let mut events = Events::new(EventSettings::new());

    let mut chess_controller = chess_controller::ChessController::new();

    while let Some(e) = events.next(&mut window) {
        chess_controller.event([600, 600], &e);

        if let Some(args) = e.render_args() {
            chess_renderer.render(&args, &chess_controller);
        }

        if let Some(args) = e.update_args() {
            chess_controller.update(&args);
            chess_renderer.update(&args);
        }
    }
}
