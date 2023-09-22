use std::path::Path;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};

use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use fritiofr_chess::{Color, Game, Move, Piece, PieceType};

mod chess_controller;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    textures: ChessTextures,
}

pub struct ChessTextures {
    white_pawn: Texture,
    white_rook: Texture,
    white_knight: Texture,
    white_bishop: Texture,
    white_queen: Texture,
    white_king: Texture,
    black_pawn: Texture,
    black_rook: Texture,
    black_knight: Texture,
    black_bishop: Texture,
    black_queen: Texture,
    black_king: Texture,
}

impl ChessTextures {
    fn piece_to_texture(&self, piece: &Piece) -> &Texture {
        match piece {
            Piece {
                color: Color::Black,
                piece_type,
            } => match piece_type {
                PieceType::Pawn => &self.black_pawn,
                PieceType::Rook => &self.black_rook,
                PieceType::Knight => &self.black_knight,
                PieceType::Bishop => &self.black_bishop,
                PieceType::Queen => &self.black_queen,
                PieceType::King => &self.black_king,
            },
            Piece {
                color: Color::White,
                piece_type,
            } => match piece_type {
                PieceType::Pawn => &self.white_pawn,
                PieceType::Rook => &self.white_rook,
                PieceType::Knight => &self.white_knight,
                PieceType::Bishop => &self.white_bishop,
                PieceType::Queen => &self.white_queen,
                PieceType::King => &self.white_king,
            },
        }
    }
}

impl App {
    fn new(gl: GlGraphics) -> App {
        macro_rules! texture {
            ($path:expr) => {
                Texture::from_path(
                    Path::new(&format!("assets/kiwen-suwi/png/{}.png", $path)),
                    &TextureSettings::new(),
                )
                .unwrap()
            };
        }

        App {
            gl,
            textures: ChessTextures {
                white_pawn: texture!("wP"),
                white_rook: texture!("wR"),
                white_knight: texture!("wN"),
                white_bishop: texture!("wB"),
                white_queen: texture!("wQ"),
                white_king: texture!("wK"),
                black_pawn: texture!("bP"),
                black_rook: texture!("bR"),
                black_knight: texture!("bN"),
                black_bishop: texture!("bB"),
                black_queen: texture!("bQ"),
                black_king: texture!("bK"),
            },
        }
    }

    fn render(&mut self, args: &RenderArgs, game_controller: &chess_controller::ChessController) {
        use graphics::*;

        const BLACK_SQUARE_COLOR: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
        const WHITE_SQUARE_COLOR: [f32; 4] = [0.65, 0.65, 0.65, 1.0];
        const SELECT_COLOR: [f32; 4] = [105.0 / 255.0, 148.0 / 255.0, 111.0 / 255.0, 1.0];
        const CAPTURE_COLOR: [f32; 4] = [148.0 / 255.0, 105.0 / 255.0, 111.0 / 255.0, 1.0];

        let width = args.window_size[0];

        let size = width / 8.0;
        let ellipse_size = size / 3.5;

        let square = rectangle::square(0.0, 0.0, size);
        let ellipse_square = rectangle::square(
            (size - ellipse_size) / 2.0,
            (size - ellipse_size) / 2.0,
            ellipse_size,
        );
        let image = Image::new().rect(graphics::rectangle::square(0.0, 0.0, size));

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK_SQUARE_COLOR, gl);
            for x in 0..8 {
                for y in 0..8 {
                    let x = x as f64;
                    let y = y as f64;

                    if x % 2.0 == y % 2.0 {
                        let transform = c.transform.trans(x * size, y * size);
                        rectangle(WHITE_SQUARE_COLOR, square, transform, gl);
                    }
                }
            }

            if let Some((x, y)) = game_controller.check {
                let transform = c.transform.trans((x as f64) * size, (y as f64) * size);
                rectangle(CAPTURE_COLOR, square, transform, gl);
            }

            if let Some((x, y)) = game_controller.from {
                let transform = c.transform.trans((x as f64) * size, (y as f64) * size);
                rectangle(SELECT_COLOR, square, transform, gl);
            }

            game_controller.moves.iter().for_each(|mv| {
                if mv.is_capture() {
                    let (x, y) = mv.to();

                    let transform = c.transform.trans((x as f64) * size, (y as f64) * size);
                    rectangle(CAPTURE_COLOR, square, transform, gl);
                } else {
                    let (x, y) = mv.to();
                    let x = x as f64;
                    let y = y as f64;
                    let transform = c.transform.trans(x * size, y * size);
                    ellipse(SELECT_COLOR, ellipse_square, transform, gl);
                }
            });

            for x in 0..8 {
                for y in 0..8 {
                    let x = x as f64;
                    let y = y as f64;

                    if let Some(piece) = game_controller
                        .game
                        .get_board()
                        .get_tile(x as usize, y as usize)
                    {
                        image.draw(
                            self.textures.piece_to_texture(&piece),
                            &graphics::draw_state::DrawState::default(),
                            c.transform.trans(x * size, y * size),
                            gl,
                        );
                    }
                }
            }

            // Draw a box rotating around the middle of the screen.
        });
    }

    fn update(&mut self, args: &UpdateArgs) {

        // Rotate 2 radians per second.
        // self.rotation += 2.0 * args.dt;
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("Chess", [600, 600])
        .graphics_api(opengl)
        .exit_on_esc(false)
        .resizable(false)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App::new(GlGraphics::new(opengl));
    let mut events = Events::new(EventSettings::new());

    let mut game_controller = chess_controller::ChessController::new();

    while let Some(e) = events.next(&mut window) {
        game_controller.event([600, 600], &e);

        if let Some(args) = e.render_args() {
            app.render(&args, &game_controller);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
