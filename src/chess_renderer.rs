use std::path::Path;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, Texture, TextureSettings};

use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use piston_window::{Glyphs, TextureContext};
use ChessAPI::piece::*;

use crate::animation::Animation;
use crate::chess_controller;

pub struct ChessRenderer {
    gl: GlGraphics, // OpenGL drawing backend.
    glyphs_regular: GlyphCache<'static>,
    glyphs_medium: GlyphCache<'static>,
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

impl ChessRenderer {
    pub fn new(gl: GlGraphics) -> ChessRenderer {
        macro_rules! texture {
            ($path:expr) => {
                Texture::from_path(
                    Path::new(&format!("assets/kiwen-suwi/png/{}.png", $path)),
                    &TextureSettings::new(),
                )
                .unwrap()
            };
        }

        let mut glyphs_regular = GlyphCache::new(
            "assets/regular.ttf",
            (),
            TextureSettings::new().filter(Filter::Nearest),
        )
        .expect("Could not load font");

        let mut glyphs_medium = GlyphCache::new(
            "assets/medium.ttf",
            (),
            TextureSettings::new().filter(Filter::Nearest),
        )
        .expect("Could not load font");

        ChessRenderer {
            gl,
            glyphs_regular,
            glyphs_medium,
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

    pub fn render(
        &mut self,
        args: &RenderArgs,
        chess_controller: &chess_controller::ChessController,
    ) {
        use graphics::*;

        const BLACK_SQUARE_COLOR: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
        const WHITE_SQUARE_COLOR: [f32; 4] = [0.65, 0.65, 0.65, 1.0];
        const SELECT_COLOR: [f32; 4] = [105.0 / 255.0, 148.0 / 255.0, 111.0 / 255.0, 1.0];
        const CAPTURE_COLOR: [f32; 4] = [148.0 / 255.0, 105.0 / 255.0, 111.0 / 255.0, 1.0];
        const LAST_MOVE_COLOR: [f32; 4] = [247.0 / 255.0, 233.0 / 255.0, 121.0 / 255.0, 0.5];

        let width = args.window_size[0];

        let size = width / 8.0;
        let ellipse_size = size / 3.5;

        let screen = rectangle::square(0.0, 0.0, width);
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
                    let x_f = x as f64;
                    let y_f = y as f64;

                    if x_f % 2.0 == y_f % 2.0 {
                        let transform = c.transform.trans(x_f * size, y_f * size);
                        rectangle(WHITE_SQUARE_COLOR, square, transform, gl);
                    }
                    if Some((x, y)) == chess_controller.last_to
                        || Some((x, y)) == chess_controller.last_from
                    {
                        let transform = c.transform.trans((x_f as f64) * size, (y_f as f64) * size);
                        rectangle(LAST_MOVE_COLOR, square, transform, gl);
                    }
                }
            }

            if let Some((x, y)) = chess_controller.check {
                let transform = c.transform.trans((x as f64) * size, (y as f64) * size);
                rectangle(CAPTURE_COLOR, square, transform, gl);
            }

            if let Some((x, y)) = chess_controller.from {
                let transform = c.transform.trans((x as f64) * size, (y as f64) * size);
                rectangle(SELECT_COLOR, square, transform, gl);
            }

            chess_controller.moves.iter().for_each(|mv| {
                if chess_controller.board.get_board()[mv.to.row as usize][mv.to.col as usize]
                    .is_some()
                {
                    let x = mv.to.col as f64;
                    let y = mv.to.row as f64;

                    let transform = c.transform.trans((x as f64) * size, (y as f64) * size);
                    rectangle(CAPTURE_COLOR, square, transform, gl);
                } else {
                    let x = mv.to.col as f64;
                    let y = mv.to.row as f64;
                    let transform = c.transform.trans(x * size, y * size);
                    ellipse(SELECT_COLOR, ellipse_square, transform, gl);
                }
            });

            for x in 0..8 {
                for y in 0..8 {
                    let x = x as f64;
                    let y = y as f64;

                    if !chess_controller.animations.is_empty() {
                        if chess_controller
                            .animations
                            .iter()
                            .any(|a| (x, y) == (a.end.0, a.end.1))
                        {
                            continue;
                        }
                    }

                    if let Some(piece) = chess_controller.board.get_board()[y as usize][x as usize]
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

            if !chess_controller.animations.is_empty() {
                for animation in chess_controller.animations.clone() {
                    let (x, y) = animation.pos();
                    let (end_x, end_y) = animation.end;

                    let piece =
                        chess_controller.board.get_board()[end_y as usize][end_x as usize].unwrap();

                    image.draw(
                        self.textures.piece_to_texture(&piece),
                        &graphics::draw_state::DrawState::default(),
                        c.transform.trans(x * size, y * size),
                        gl,
                    );
                }
            }

            // Draw the promotion dialog box thingy
            {
                let t = chess_controller.promotion_animation.value() as f32;
                let t = match chess_controller.promotion_dialog {
                    true => t,
                    false => 1.0 - t,
                };
                rectangle([0.0, 0.0, 0.0, 0.9 * t], screen, c.transform, gl);

                let rect = rectangle::Rectangle::new_round([0.95, 0.95, 0.95, 1.0 * t], 5.0);

                for i in 0..4 {
                    let gap = 10.0;

                    let x = width / 2.0 - (size + gap) * 2.0 + gap / 2.0 + (size + gap) * i as f64;
                    let y = width / 2.0 - size / 2.0;
                    let trans = c.transform.trans(x, y);

                    rect.draw(square, &Default::default(), trans, gl);
                    let tex = self.textures.piece_to_texture(&Piece {
                        color: chess_controller.promotion_color,
                        piece_type: match i {
                            0 => PieceType::Queen,
                            1 => PieceType::Rook,
                            2 => PieceType::Bishop,
                            3 => PieceType::Knight,
                            _ => unreachable!(),
                        },
                    });

                    let image = image.clone().color([1.0, 1.0, 1.0, 1.0 * t]);
                    image.draw(tex, &graphics::draw_state::DrawState::default(), trans, gl);
                }
            }

            if let Some(state) = chess_controller.end_state {
                let t = chess_controller.end_state_animation.value() as f32;

                let t = match chess_controller.end_state_show {
                    true => t,
                    false => 1.0 - t,
                };

                rectangle([0.0, 0.0, 0.0, 0.9 * t], screen, c.transform, gl);
                let rect = rectangle::Rectangle::new_round([0.95, 0.95, 0.95, 1.0 * t], 5.0);

                rect.draw(
                    rectangle::centered([width / 2.0, width / 2.0, width / 3.0, width / 10.0]),
                    &Default::default(),
                    c.transform,
                    gl,
                );

                let text = Text::new_color([0.0, 0.0, 0.0, 1.0 * t], 32);

                let black_won_offset = 7.1;
                let white_won_offset = 7.05;
                let stalemate_offset = 7.23;

                let (offset, content) = match state {
                    2 => (white_won_offset, "White won"),
                    1 => (black_won_offset, "Black won"),
                    0 => (stalemate_offset, "Stalemate"),
                    _ => unreachable!(),
                };

                text.draw_pos(
                    content,
                    [width / 2.0, width / 2.0],
                    &mut self.glyphs_medium,
                    &Default::default(),
                    c.transform.trans(-width / offset, 10.0),
                    gl,
                )
                .unwrap();
            }
        });
    }

    pub fn update(&mut self, args: &UpdateArgs) {}
}
