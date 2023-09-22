use fritiofr_chess::PieceType;
use fritiofr_chess::{Game, Move};
use piston::input::*;
use piston::Event;

pub struct ChessController {
    pub game: Game,
    pub from: Option<(usize, usize)>,
    pub moves: Vec<Move>,
    pub check: Option<(usize, usize)>,
    cursor_pos: [f64; 2],
}

impl ChessController {
    pub fn new() -> ChessController {
        ChessController {
            game: Game::start_pos(),
            from: None,
            check: None,
            moves: Vec::new(),
            cursor_pos: [0.0, 0.0],
        }
    }

    pub fn event(&mut self, size: [u32; 2], e: &Event) {
        if let Some(pos) = e.mouse_cursor_args() {
            self.cursor_pos = pos;
        }

        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            let x = (self.cursor_pos[0]) / ((size[0] as f64) / 8.0);
            let y = (self.cursor_pos[1]) / ((size[1] as f64) / 8.0);

            let x = x as usize;
            let y = y as usize;

            let mv = self.moves.iter().find(|mv| mv.to() == (x, y));

            if let Some(mv) = mv {
                self.game.apply_move(*mv).unwrap();

                if self.game.is_check() {
                    self.check = Some(
                        (0..8)
                            .map(|x| (0..8).map(move |y| (x, y)))
                            .flatten()
                            .find(|&(x, y)| {
                                if let Some(piece) = self.game.get_board().get_tile(x, y) {
                                    return piece.color == self.game.get_turn()
                                        && piece.piece_type == PieceType::King;
                                } else {
                                    return false;
                                }
                            })
                            .unwrap(),
                    );
                } else {
                    self.check = None;
                }

                self.from = None;
                self.moves = Vec::new();
            } else if let Some(moves) = self.game.gen_moves(x, y) {
                self.from = Some((x as usize, y as usize));
                self.moves = moves;
            } else {
                self.from = None;
                self.moves = Vec::new();
            }
        }
    }
}
